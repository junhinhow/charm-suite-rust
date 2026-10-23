//! Behavioural tests ported from the Go suite (`store_test.go`, `select_test.go`
//! and `example_test.go`).
//!
//! Cases that depend on Go `nil` arguments (`Update(nil)`, `Subscribe(nil)`,
//! `Snapshot(nil)`, `Restore(nil)`, `Select(nil, ...)`) do not exist in Rust:
//! the type system rejects them, so they are not ported.

use std::io;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use charm_store::{select, Codec, CodecError, Error, JsonCodec, Store};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct EstadoSnapshot {
    nome: String,
    itens: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct EstadoSelecionado {
    contador: i32,
    nome: String,
    tags: Vec<String>,
}

fn recorder<T: Clone + Send + 'static>(s: &Store<T>) -> Arc<Mutex<Vec<T>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let _cancel = s.subscribe(move |v: T| sink.lock().unwrap().push(v));
    seen
}

fn counter(s: &Store<EstadoSnapshot>) -> Arc<AtomicUsize> {
    let calls = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&calls);
    let _cancel = s.subscribe(move |_: EstadoSnapshot| {
        sink.fetch_add(1, Ordering::SeqCst);
    });
    calls
}

struct FailingCodec;

impl Codec for FailingCodec {
    fn marshal<V: Serialize + ?Sized>(&self, _value: &V) -> Result<Vec<u8>, CodecError> {
        Err(Box::new(io::Error::other("falha proposital")))
    }

    fn unmarshal<V: DeserializeOwned>(&self, _data: &[u8]) -> Result<V, CodecError> {
        Err(Box::new(io::Error::other("falha proposital")))
    }
}

#[test]
fn get_set_update() {
    let s = Store::new(1);
    assert_eq!(s.get(), 1);

    s.set(2);
    assert_eq!(s.get(), 2);

    s.update(|v| v + 3);
    assert_eq!(s.get(), 5);
}

#[test]
fn set_igual_nao_notifica_nem_empilha() {
    let inicial = EstadoSnapshot {
        nome: "a".to_string(),
        itens: vec!["x".to_string()],
    };
    let s = Store::new(inicial);
    let calls = counter(&s);

    s.set(EstadoSnapshot {
        nome: "a".to_string(),
        itens: vec!["x".to_string()],
    });
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(s.history(), 0);

    s.update(|v| v);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(s.history(), 0);

    s.set(EstadoSnapshot {
        nome: "b".to_string(),
        itens: vec!["x".to_string()],
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(s.history(), 1);
}

#[test]
fn multiplos_inscritos_e_ordem() {
    let s = Store::new(0);
    let ordem = Arc::new(Mutex::new(Vec::new()));

    for nome in ["primeiro", "segundo", "terceiro"] {
        let sink = Arc::clone(&ordem);
        let _cancel = s.subscribe(move |_: i32| sink.lock().unwrap().push(nome));
    }

    s.notify();
    assert_eq!(
        *ordem.lock().unwrap(),
        vec!["primeiro", "segundo", "terceiro"]
    );
}

#[test]
fn unsubscribe_e_idempotente() {
    let s = Store::new(0);
    let chamadas = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&chamadas);
    let cancelar = s.subscribe(move |_: i32| {
        sink.fetch_add(1, Ordering::SeqCst);
    });
    cancelar();
    cancelar();
    cancelar();

    s.notify();
    assert_eq!(chamadas.load(Ordering::SeqCst), 0);
}

#[test]
fn unsubscribe_durante_notificacao() {
    type Slot = Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>;

    let s = Store::new(0);
    let chamadas: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let cancelar_b: Slot = Arc::new(Mutex::new(None));

    let slot = Arc::clone(&cancelar_b);
    let sink = Arc::clone(&chamadas);
    let _cancel_a = s.subscribe(move |_: i32| {
        sink.lock().unwrap().push("a");
        if let Some(cancel) = slot.lock().unwrap().as_ref() {
            cancel();
            cancel();
        }
    });
    let cancel_b = s.subscribe({
        let sink = Arc::clone(&chamadas);
        move |_: i32| sink.lock().unwrap().push("b")
    });
    *cancelar_b.lock().unwrap() = Some(Box::new(cancel_b));

    s.notify();
    assert_eq!(*chamadas.lock().unwrap(), vec!["a", "b"]);

    s.notify();
    assert_eq!(*chamadas.lock().unwrap(), vec!["a", "b", "a"]);
}

#[test]
fn notify_sempre_dispara_com_valor_atual() {
    let s = Store::new(7);
    let recebidos = recorder(&s);

    s.notify();
    s.notify();
    assert_eq!(*recebidos.lock().unwrap(), vec![7, 7]);
}

#[test]
fn undo_redo() {
    let s = Store::new("a".to_string());
    assert!(!s.undo());
    assert!(!s.redo());

    let recebidos = recorder(&s);

    s.set("b".to_string());
    s.set("c".to_string());
    assert!(s.can_undo());
    assert!(!s.can_redo());
    assert_eq!(s.history(), 2);

    assert!(s.undo());
    assert_eq!(s.get(), "b");
    assert_eq!(s.history(), 1);
    assert!(s.can_redo());

    assert!(s.undo());
    assert_eq!(s.get(), "a");
    assert!(!s.undo());

    assert!(s.redo());
    assert_eq!(s.get(), "b");
    assert!(s.redo());
    assert!(!s.redo());
    assert_eq!(s.get(), "c");

    assert_eq!(
        *recebidos.lock().unwrap(),
        vec![
            "b".to_string(),
            "c".to_string(),
            "b".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]
    );
}

#[test]
fn set_limpa_redo() {
    let s = Store::new(0);
    s.set(1);
    s.set(2);
    assert!(s.undo());
    assert!(s.can_redo());

    s.set(5);
    assert!(!s.can_redo());

    s.set(5);
    assert_eq!(s.history(), 2);
}

#[test]
fn history_limit_um() {
    let s = Store::new(0);
    s.set_history_limit(1);
    s.set(1);
    s.set(2);
    s.set(3);
    assert_eq!(s.history(), 1);

    assert!(s.undo());
    assert_eq!(s.get(), 2);
    assert!(!s.undo());

    assert!(s.redo());
    assert_eq!(s.get(), 3);
}

#[test]
fn history_limit_padrao_e_reducao() {
    let s = Store::new(0);
    for i in 1..=101 {
        s.set(i);
    }
    assert_eq!(s.history(), 100);

    s.set_history_limit(3);
    assert_eq!(s.history(), 3);

    assert!(s.undo());
    assert_eq!(s.get(), 100);
    assert!(s.undo());
    assert_eq!(s.get(), 99);
    assert!(s.undo());
    assert_eq!(s.get(), 98);
    assert!(!s.undo());

    s.set(200);
    s.set_history_limit(0);
    assert_eq!(s.history(), 1);
}

#[test]
fn set_history_limit_nao_notifica() {
    let s = Store::new(0);
    let calls = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&calls);
    let _cancel = s.subscribe(move |_: i32| {
        sink.fetch_add(1, Ordering::SeqCst);
    });

    s.set_history_limit(10);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn snapshot_restore_json() {
    let origem = Store::new(EstadoSnapshot::default());
    origem.set(EstadoSnapshot {
        nome: "primeiro".to_string(),
        itens: vec!["a".to_string()],
    });
    origem.set(EstadoSnapshot {
        nome: "segundo".to_string(),
        itens: vec!["b".to_string(), "c".to_string()],
    });

    let data = origem.snapshot(&JsonCodec).expect("snapshot");
    assert_ne!(data.len(), 0);

    let destino = Store::new(EstadoSnapshot::default());
    destino.set(EstadoSnapshot {
        nome: "antigo".to_string(),
        itens: Vec::new(),
    });
    assert_eq!(destino.history(), 1);

    let notificados = recorder(&destino);
    destino.restore(&JsonCodec, &data).expect("restore");

    assert_eq!(destino.get(), origem.get());
    assert_eq!(destino.history(), 0);
    assert!(!destino.can_undo());
    assert!(!destino.can_redo());
    assert_eq!(*notificados.lock().unwrap(), vec![origem.get()]);
}

#[test]
fn restore_notifica_mesmo_com_valor_igual() {
    let s = Store::new(EstadoSnapshot {
        nome: "igual".to_string(),
        itens: Vec::new(),
    });
    let calls = counter(&s);

    let data = s.snapshot(&JsonCodec).expect("snapshot");
    s.restore(&JsonCodec, &data).expect("restore");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn erros_de_codec() {
    let s = Store::new(10);

    let err = s.snapshot(&FailingCodec).expect_err("snapshot must fail");
    assert!(matches!(err, Error::Snapshot(_)));
    assert!(err.to_string().contains("snapshot"));
    assert_eq!(
        std::error::Error::source(&err).map(ToString::to_string),
        Some("falha proposital".to_string())
    );

    let err = s
        .restore(&FailingCodec, b"qualquer")
        .expect_err("restore must fail");
    assert!(matches!(err, Error::Restore(_)));
    assert!(err.to_string().contains("restore"));
    assert_eq!(
        std::error::Error::source(&err).map(ToString::to_string),
        Some("falha proposital".to_string())
    );
}

#[test]
fn restore_invalido_nao_altera_estado() {
    let s = Store::new(EstadoSnapshot {
        nome: "atual".to_string(),
        itens: Vec::new(),
    });
    s.set(EstadoSnapshot {
        nome: "anterior".to_string(),
        itens: Vec::new(),
    });
    let antes = s.get();
    let history_antes = s.history();
    let calls = counter(&s);

    let err = s
        .restore(&JsonCodec, b"{")
        .expect_err("invalid JSON must fail");
    assert!(err.to_string().contains("restore"));
    assert_eq!(s.get(), antes);
    assert_eq!(s.history(), history_antes);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn concorrencia() {
    const THREADS: i64 = 50;
    const ITERACOES: i64 = 20;

    let s = Store::new(0_i64);
    let chamadas = Arc::new(AtomicI64::new(0));

    std::thread::scope(|scope| {
        for i in 0..THREADS {
            let s = &s;
            let chamadas = Arc::clone(&chamadas);
            scope.spawn(move || {
                for j in 0..ITERACOES {
                    s.set(i * ITERACOES + j);
                    let _ = s.get();

                    let sink = Arc::clone(&chamadas);
                    let cancelar = s.subscribe(move |_: i64| {
                        sink.fetch_add(1, Ordering::SeqCst);
                    });
                    s.notify();
                    cancelar();
                    cancelar();

                    s.update(|v| v + 1);
                    s.undo();
                    s.redo();
                    let _ = s.can_undo();
                    let _ = s.can_redo();
                    let _ = s.history();
                }
            });
        }
    });

    assert!(s.get() >= 0);
    assert!(chamadas.load(Ordering::SeqCst) > 0);
}

#[test]
fn select_propaga_apenas_mudancas() {
    let origem = Store::new(EstadoSelecionado {
        contador: 1,
        nome: "a".to_string(),
        tags: Vec::new(),
    });
    let contador = select(&origem, |e: EstadoSelecionado| e.contador);
    assert_eq!(contador.get(), 1);

    let recebidos = recorder(&contador);

    origem.set(EstadoSelecionado {
        contador: 1,
        nome: "b".to_string(),
        tags: Vec::new(),
    });
    assert!(recebidos.lock().unwrap().is_empty());

    origem.set(EstadoSelecionado {
        contador: 2,
        nome: "b".to_string(),
        tags: Vec::new(),
    });
    assert_eq!(*recebidos.lock().unwrap(), vec![2]);
    assert_eq!(contador.get(), 2);

    origem.notify();
    assert_eq!(*recebidos.lock().unwrap(), vec![2]);

    origem.set(EstadoSelecionado {
        contador: 2,
        nome: "c".to_string(),
        tags: Vec::new(),
    });
    assert_eq!(*recebidos.lock().unwrap(), vec![2]);
}

#[test]
fn select_compara_com_deep_equal() {
    let origem = Store::new(EstadoSelecionado {
        contador: 0,
        nome: String::new(),
        tags: vec!["a".to_string(), "b".to_string()],
    });
    let tags = select(&origem, |e: EstadoSelecionado| e.tags);
    let recebidos = recorder(&tags);

    origem.set(EstadoSelecionado {
        contador: 0,
        nome: String::new(),
        tags: vec!["a".to_string(), "b".to_string()],
    });
    assert!(recebidos.lock().unwrap().is_empty());

    origem.set(EstadoSelecionado {
        contador: 0,
        nome: String::new(),
        tags: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    });
    assert_eq!(
        *recebidos.lock().unwrap(),
        vec![vec!["a".to_string(), "b".to_string(), "c".to_string()]]
    );
}

#[test]
fn select_nao_tem_historico() {
    let origem = Store::new(0);
    let dobro = select(&origem, |v: i32| v * 2);

    origem.set(1);
    origem.set(2);

    assert_eq!(dobro.get(), 4);
    assert_eq!(dobro.history(), 0);
    assert!(!dobro.can_undo());
    assert!(!dobro.can_redo());
    assert!(!dobro.undo());
    assert!(!dobro.redo());
}

#[test]
fn example_store() {
    let s = Store::new(0);
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    let _cancel = s.subscribe(move |v: i32| sink.lock().unwrap().push(format!("mudou: {v}")));

    s.set(1);
    s.set(1);
    s.update(|v| v + 1);
    log.lock().unwrap().push(format!("valor: {}", s.get()));

    s.undo();
    log.lock().unwrap().push(format!("após undo: {}", s.get()));

    assert_eq!(
        *log.lock().unwrap(),
        vec![
            "mudou: 1",
            "mudou: 2",
            "valor: 2",
            "mudou: 1",
            "após undo: 1"
        ]
    );
}

#[test]
fn example_select() {
    let origem = Store::new(vec![1, 2, 3]);
    let tamanho = select(&origem, |v: Vec<i32>| v.len());
    let recebidos = recorder(&tamanho);

    origem.set(vec![1, 2]);
    origem.set(vec![1, 2, 3, 4]);

    assert_eq!(*recebidos.lock().unwrap(), vec![2, 4]);
}
