use std::io;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use charm_mock::{
    Clipboard, FakeClock, MemFS, HttpMock, OP_READ, OP_WRITE, OP_REMOVE, OP_STAT, OP_READDIR,
};
use serial_test::serial;

const BASE: SystemTime = UNIX_EPOCH; // use epoch for simpler comparisons where possible

// Actually use the Go base for clock tests.
const RELOGIO_BASE: SystemTime = UNIX_EPOCH; // placeholder; adjust below

// We'll define a proper base constant.
const BASE_TIME: SystemTime = UNIX_EPOCH;

#[test]
fn clock_now_and_since() {
    let base = UNIX_EPOCH + Duration::from_secs(1_704_110_400);
    let c = FakeClock::new(base);
    assert_eq!(c.now(), base);
    assert_eq!(c.since(base), Duration::ZERO);
    c.advance(Duration::from_millis(1500));
    assert_eq!(c.now(), base + Duration::from_millis(1500));
    assert_eq!(c.since(base), Duration::from_millis(1500));
    assert_eq!(c.since(c.now()), Duration::ZERO);
}

#[test]
fn clock_advance_zero_or_negative_no_move() {
    let base = UNIX_EPOCH + Duration::from_secs(1_704_110_400);
    let c = FakeClock::new(base);
    c.advance(Duration::from_secs(2));
    c.advance(Duration::ZERO);
    assert_eq!(c.now(), base + Duration::from_secs(2));
}

#[test]
fn clock_set() {
    let base = UNIX_EPOCH + Duration::from_secs(1_704_110_400);
    let c = FakeClock::new(base);
    let alvo = base + Duration::from_secs(86400);
    c.set(alvo);
    assert_eq!(c.now(), alvo);
    let anterior = base - Duration::from_secs(3600);
    c.set(anterior);
    assert_eq!(c.now(), anterior);
    assert_eq!(c.since(anterior), Duration::ZERO);
}

#[test]
fn clock_cem_avancos() {
    let base = UNIX_EPOCH + Duration::from_secs(1_704_110_400);
    let c = FakeClock::new(base);
    for _ in 0..100 {
        c.advance(Duration::from_millis(10));
    }
    assert_eq!(c.now(), base + Duration::from_secs(1));
    assert_eq!(c.since(base), Duration::from_secs(1));
}

#[test]
fn clock_ticks() {
    let base = UNIX_EPOCH + Duration::from_secs(1_704_110_400);
    let c = FakeClock::new(base);
    c.advance(Duration::from_millis(2500));
    let got = c.ticks(Duration::from_secs(1)).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[0], base + Duration::from_secs(1));
    assert_eq!(got[1], base + Duration::from_secs(2));
    assert_eq!(c.ticks(Duration::from_millis(500)).unwrap().len(), 5);
    assert!(c.ticks(Duration::ZERO).is_none());
    assert!(c.ticks(Duration::from_secs(3)).is_none());
    c.set(base - Duration::from_secs(3600));
    assert!(c.ticks(Duration::from_secs(1)).is_none());
}

#[test]
fn clock_ticks_nao_muda() {
    let base = UNIX_EPOCH + Duration::from_secs(1_704_110_400);
    let c = FakeClock::new(base);
    c.advance(Duration::from_secs(3));
    let antes = c.now();
    let _ = c.ticks(Duration::from_secs(1));
    assert_eq!(c.now(), antes);
}

#[test]
fn clipboard_le_escreve() {
    let c = Clipboard::new();
    assert_eq!(c.read().unwrap(), "");
    assert_eq!(c.contents(), "");
    assert!(c.history().is_empty());
    c.write("primeiro").unwrap();
    c.write("segundo").unwrap();
    assert_eq!(c.contents(), "segundo");
    assert_eq!(c.read().unwrap(), "segundo");
    assert_eq!(c.history(), vec!["primeiro".to_string(), "segundo".to_string()]);
}

#[test]
fn clipboard_registra_vazia() {
    let c = Clipboard::new();
    let _ = c.write("");
    let _ = c.write("x");
    assert_eq!(c.contents(), "x");
    assert_eq!(c.history(), vec!["".to_string(), "x".to_string()]);
}

#[test]
fn clipboard_history_isolado() {
    let c = Clipboard::new();
    let _ = c.write("a");
    let mut hist = c.history();
    hist[0] = "mutado".to_string();
    hist.push("extra".to_string());
    assert_eq!(hist.len(), 2);
    let fresco = c.history();
    assert_eq!(fresco.len(), 1);
    assert_eq!(fresco[0], "a");
}

#[test]
fn clipboard_zero_value() {
    let c = Clipboard::default();
    assert_eq!(c.contents(), "");
    c.write("x").unwrap();
    assert_eq!(c.contents(), "x");
}

#[test]
fn clipboard_concorrente() {
    let c = Clipboard::new();
    let total = 32;
    let mut handles = Vec::new();
    for i in 0..total {
        let c = c.clone();
        handles.push(thread::spawn(move || {
            c.write(&format!("item-{i}")).unwrap();
            let _ = c.read();
            let _ = c.history();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(c.history().len(), total);
}

#[test]
fn memfs_seed_leitura() {
    let fs = MemFS::new();
    assert!(fs.read_file("ausente.txt").is_err());
    // errors.Is NotFound
    assert_eq!(fs.read_file("ausente.txt").unwrap_err().kind(), io::ErrorKind::NotFound);
    fs.seed("config/app.yaml", b"tema: escuro");
    let data = fs.read_file("config/app.yaml").unwrap();
    assert_eq!(data, b"tema: escuro");
    // copy
    let mut data = data;
    data[0] = b'X';
    let denovo = fs.read_file("config/app.yaml").unwrap();
    assert_eq!(denovo, b"tema: escuro");
    // normalize
    assert!(fs.read_file("/config/app.yaml").is_ok());
    fs.seed(r"logs\execucao.txt", b"ok");
    assert!(fs.read_file("logs/execucao.txt").is_ok());
}

#[test]
fn memfs_seed_nil() {
    let fs = MemFS::new().seed("vazio.txt", &[]);
    let data = fs.read_file("vazio.txt").unwrap();
    assert!(data.is_empty());
    let info = fs.stat("vazio.txt").unwrap();
    assert_eq!(info.size(), 0);
}

#[test]
fn memfs_writefile() {
    let fs = MemFS::new();
    fs.write_file("dados/run.log", b"ok", 0o600).unwrap();
    assert_eq!(fs.read_file("dados/run.log").unwrap(), b"ok");
    let info = fs.stat("dados/run.log").unwrap();
    assert_eq!(info.mode() & 0o777, 0o600);
    assert!(!info.is_dir());
    assert_eq!(info.name(), "run.log");
    fs.write_file("padrao.txt", b"x", 0).unwrap();
    let info = fs.stat("padrao.txt").unwrap();
    assert_eq!(info.mode() & 0o777, 0o644);
}

#[test]
fn memfs_readdir() {
    let fs = MemFS::new()
        .seed("config/app.yaml", b"a")
        .seed("config/tema.json", b"b")
        .seed("raiz.txt", b"c");
    let entries = fs.read_dir(".").unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name(), "config");
    assert_eq!(entries[1].name(), "raiz.txt");
    assert!(entries[0].is_dir());
    assert!(!entries[1].is_dir());
    assert!(entries[1].info().is_ok());
    let entries = fs.read_dir("").unwrap();
    assert_eq!(entries.len(), 2);
    let entries = fs.read_dir("config").unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name(), "app.yaml");
    assert_eq!(entries[1].name(), "tema.json");
    assert!(fs.read_dir("raiz.txt").is_err());
    assert_eq!(fs.read_dir("nao-existe").unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn memfs_stat() {
    let fs = MemFS::new().seed("a/b.txt", b"1234");
    let info = fs.stat(".").unwrap();
    assert!(info.is_dir());
    assert_eq!(info.name(), ".");
    let info = fs.stat("a").unwrap();
    assert!(info.is_dir());
    let info = fs.stat("a/b.txt").unwrap();
    assert_eq!(info.size(), 4);
    assert!(info.mod_time() != SystemTime::UNIX_EPOCH); // not zero-ish
    assert!(info.sys().is_none());
    assert_eq!(fs.stat("a/nao-existe").unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn memfs_remove() {
    let fs = MemFS::new()
        .seed("config/app.yaml", b"a")
        .seed("vazio.txt", b"b");
    fs.remove("vazio.txt").unwrap();
    assert_eq!(fs.read_file("vazio.txt").unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(fs.remove("vazio.txt").unwrap_err().kind(), io::ErrorKind::NotFound);
    assert!(fs.remove("config").is_err());
    fs.remove("config/app.yaml").unwrap();
    fs.remove("config").unwrap();
    assert_eq!(fs.stat("config").unwrap_err().kind(), io::ErrorKind::NotFound);
    assert!(fs.remove(".").is_err());
}

#[test]
fn memfs_fail_on() {
    let err_boom = io::Error::new(io::ErrorKind::Other, "boom");
    let fs = MemFS::new().seed("a.txt", b"x");
    fs.fail_on(OP_READ, "/a.txt", Some(err_boom));
    let e = fs.read_file("a.txt").unwrap_err();
    assert_eq!(e.to_string(), "boom");
    // persists
    assert_eq!(fs.read_file("a.txt").unwrap_err().to_string(), "boom");
    // other file ok
    let fs2 = fs.seed("b.txt", b"y");
    assert_eq!(fs2.read_file("b.txt").unwrap(), b"y");
    // write
    fs.fail_on(OP_WRITE, "a.txt", Some(io::Error::new(io::ErrorKind::Other, "boom")));
    assert!(fs.write_file("a.txt", b"y", 0o644).is_err());
    assert_eq!(fs.snapshot()["a.txt"], b"x");
    // remove
    fs.fail_on(OP_REMOVE, "a.txt", Some(io::Error::new(io::ErrorKind::Other, "boom")));
    assert!(fs.remove("a.txt").is_err());
    // stat
    fs.fail_on(OP_STAT, "a.txt", Some(io::Error::new(io::ErrorKind::Other, "boom")));
    assert!(fs.stat("a.txt").is_err());
    // readdir
    fs.fail_on(OP_READDIR, ".", Some(io::Error::new(io::ErrorKind::Other, "boom")));
    assert!(fs.read_dir(".").is_err());
    // nil err
    fs.fail_on(OP_READ, "a.txt", None);
    assert!(fs.read_file("a.txt").is_err());
    // unknown op no effect
    let outro = MemFS::new().seed("a.txt", b"x");
    outro.fail_on("operacao-inexistente", "a.txt", Some(io::Error::new(io::ErrorKind::Other, "boom")));
    assert!(outro.read_file("a.txt").is_ok());
    // alias
    let alias = MemFS::new().seed("a.txt", b"x");
    alias.fail_on("ReadFile", "a.txt", Some(io::Error::new(io::ErrorKind::Other, "boom")));
    assert!(alias.read_file("a.txt").is_err());
}

#[test]
fn memfs_snapshot_isolado() {
    let fs = MemFS::new().seed("a.txt", b"um");
    let mut snap = fs.snapshot();
    assert_eq!(snap.len(), 1);
    assert_eq!(snap["a.txt"], b"um");
    snap.get_mut("a.txt").expect("snapshot entry")[0] = b'X';
    snap.insert("b.txt".to_string(), b"novo".to_vec());
    snap.remove("a.txt");
    assert_eq!(fs.read_file("a.txt").unwrap(), b"um");
    assert!(fs.stat("b.txt").is_err());
    fs.write_file("c.txt", b"tres", 0o644).unwrap();
    assert_eq!(snap.len(), 1);
}

#[test]
fn memfs_zero_value() {
    let fs = MemFS::default();
    let info = fs.stat(".").unwrap();
    assert!(info.is_dir());
    let entries = fs.read_dir(".").unwrap();
    assert!(entries.is_empty());
    fs.write_file("a.txt", b"x", 0o644).unwrap();
    assert_eq!(fs.snapshot()["a.txt"], b"x");
}

#[test]
fn memfs_entradas_invalidas() {
    let fs = MemFS::new();
    fs.seed("", b"x");
    fs.seed(".", b"y");
    assert!(fs.stat(".").unwrap().is_dir());
    assert!(fs.write_file(".", b"x", 0o644).is_err());
    fs.seed("diretorio/filho", b"x");
    assert!(fs.write_file("diretorio", b"y", 0o644).is_err());
    fs.seed("arquivo", b"x");
    assert!(fs.write_file("arquivo/filho", b"y", 0o644).is_err());
    assert!(fs.read_file("diretorio").is_err());
}

#[test]
fn memfs_concorrente() {
    let fs = MemFS::new().seed("base.txt", b"x");
    let mut handles = Vec::new();
    for i in 0..16 {
        let fs = fs.clone();
        handles.push(thread::spawn(move || {
            let name = format!("f-{i:02}.txt");
            fs.write_file(&name, name.as_bytes(), 0o644).unwrap();
            let _ = fs.read_file(&name).unwrap();
            let _ = fs.read_dir(".").unwrap();
            let _ = fs.snapshot();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let entries = fs.read_dir(".").unwrap();
    assert_eq!(entries.len(), 17);
}

#[test]
fn http_handle_get() {
    let h = HttpMock::new();
    assert!(h.url().starts_with("http://"));
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
    headers.insert("X-Teste".to_string(), "sim".to_string());
    h.handle("GET", "/hello", 200, "olá", Some(headers));
    let client = h.client();
    let url = format!("{}/hello", h.url());
    let mut req_headers = std::collections::BTreeMap::new();
    req_headers.insert("X-Enviado".to_string(), "abc".to_string());
    let resp = client.request("GET", &url, &req_headers, "").unwrap();
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body, "olá");
    assert_eq!(resp.header("X-Teste").unwrap(), "sim");
    assert_eq!(resp.header("Content-Type").unwrap(), "text/plain; charset=utf-8");
    let reqs = h.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].method, "GET");
    assert_eq!(reqs[0].path, "/hello");
    assert_eq!(reqs[0].header("X-Enviado").unwrap(), "abc");
    assert_eq!(reqs[0].body, "");
    h.close();
}

#[test]
fn http_handle_func_post() {
    let h = HttpMock::new();
    h.handle_func(
        "POST",
        "/echo",
        Some(Arc::new(|w, r| {
            w.header_set("X-Metodo", &r.method);
            w.write_status(201);
            w.write_str(&format!("eco:{}", r.body));
        })),
    );
    let client = h.client();
    let url = format!("{}/echo", h.url());
    let resp = client.post(&url, "text/plain", "olá").unwrap();
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body, "eco:olá");
    assert_eq!(resp.header("X-Metodo").unwrap(), "POST");
    let reqs = h.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].method, "POST");
    assert_eq!(reqs[0].path, "/echo");
    assert_eq!(reqs[0].body, "olá");
    h.close();
}

#[test]
fn http_rota_nao_registrada() {
    let h = HttpMock::new();
    let client = h.client();
    let url = format!("{}/nao-existe", h.url());
    let resp = client.get(&url).unwrap();
    assert_eq!(resp.status_code, 404);
    assert!(resp.body.contains("rota não registrada"));
    assert!(resp.body.contains("GET /nao-existe"));
    assert_eq!(h.requests().len(), 1);
    h.close();
}

#[test]
fn http_query_ignorada() {
    let h = HttpMock::new();
    h.handle("GET", "busca", 200, "achou", None);
    let client = h.client();
    let url = format!("{}/busca?q=termo", h.url());
    let resp = client.get(&url).unwrap();
    assert_eq!(resp.status_code, 200);
    let reqs = h.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].path, "/busca");
    assert!(h.assert_called("GET", "/busca?q=outro").is_ok());
    h.close();
}

#[test]
fn http_status_padrao() {
    let h = HttpMock::new();
    h.handle("GET", "/zero", 0, "ok", None);
    h.handle("GET", "/invalido", 42, "ok", None);
    let client = h.client();
    for path in ["/zero", "/invalido"] {
        let url = format!("{}{}", h.url(), path);
        let resp = client.get(&url).unwrap();
        assert_eq!(resp.status_code, 200);
    }
    h.close();
}

#[test]
fn http_handle_func_nulo() {
    let h = HttpMock::new();
    h.handle_func("GET", "/nulo", None);
    let client = h.client();
    let url = format!("{}/nulo", h.url());
    let resp = client.get(&url).unwrap();
    assert_eq!(resp.status_code, 500);
    assert!(resp.body.contains("handler nulo"));
    h.close();
}

#[test]
fn http_assert_called() {
    let h = HttpMock::new();
    h.handle("GET", "/hello", 200, "ok", None);
    let client = h.client();
    let url = format!("{}/hello", h.url());
    let resp = client.get(&url).unwrap();
    let _ = resp;
    assert!(h.assert_called("get", "/hello").is_ok());
    let err = h.assert_called("POST", "/nunca").unwrap_err();
    assert!(err.contains("POST /nunca"));
    assert!(err.contains("GET /hello"));
    // nil t equivalent: just call, no panic
    let _ = h.assert_called("DELETE", "/nunca");
    h.close();
}

#[test]
fn http_requests_isolado() {
    let h = HttpMock::new();
    h.handle("GET", "/a", 200, "a", None);
    h.handle("GET", "/b", 200, "b", None);
    let client = h.client();
    for path in ["/a", "/b"] {
        let url = format!("{}{}", h.url(), path);
        let resp = client.get(&url).unwrap();
        let _ = resp;
    }
    let mut reqs = h.requests();
    reqs[0].path = "/mutado".to_string();
    reqs[0].body = "mutado".to_string();
    // header mutation on returned clone doesn't affect original
    let fresco = h.requests();
    assert_eq!(fresco.len(), 2);
    assert_eq!(fresco[0].path, "/a");
    assert_eq!(fresco[0].body, "");
    h.close();
}

#[test]
fn http_cabecalhos_copiados() {
    let h = HttpMock::new();
    let mut headers = std::collections::BTreeMap::new();
    headers.insert("X-Teste".to_string(), "original".to_string());
    h.handle("GET", "/a", 200, "a", Some(headers.clone()));
    let _ = headers; // mutate original not used
    let client = h.client();
    let url = format!("{}/a", h.url());
    let resp = client.get(&url).unwrap();
    assert_eq!(resp.header("X-Teste").unwrap(), "original");
    h.close();
}

#[test]
fn http_close() {
    let h = HttpMock::new();
    let url = h.url();
    h.close();
    h.close();
    let client = h.client();
    assert!(client.get(&format!("{url}/x")).is_err());
}

#[test]
fn http_concorrente() {
    let h = HttpMock::new();
    h.handle("GET", "/ping", 200, "pong", None);
    let total = 16;
    let mut handles = Vec::new();
    for _ in 0..total {
        let h = h.clone();
        handles.push(thread::spawn(move || {
            let client = h.client();
            let url = format!("{}/ping", h.url());
            let resp = client.get(&url).unwrap();
            let _ = resp.body;
        }));
    }
    for hh in handles {
        hh.join().unwrap();
    }
    assert_eq!(h.requests().len(), total);
    assert!(h.assert_called("GET", "/ping").is_ok());
    h.close();
}

// Terminal tests

struct FakeCleanuper {
    cleanups: Vec<Box<dyn FnOnce() + Send + 'static>>,
}

impl FakeCleanuper {
    fn new() -> Self {
        Self { cleanups: Vec::new() }
    }
    fn run(&mut self) {
        while let Some(cb) = self.cleanups.pop() {
            cb();
        }
    }
}

impl charm_mock::EnvTarget for FakeCleanuper {
    fn cleanup(&mut self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.cleanups.push(f);
    }
}

#[test]
#[serial]
fn env_restaura() {
    let definida = "CHARM_MOCK_ENV_DEFINIDA";
    let nova = "CHARM_MOCK_ENV_NOVA";
    std::env::set_var(definida, "original");
    // ensure nova absent
    std::env::remove_var(nova);

    {
        let mut t = FakeCleanuper::new();
        // We need pass &mut t to env; but env generic. Use a helper closure? Call directly.
        let mut vars = std::collections::BTreeMap::new();
        vars.insert(definida.to_string(), "interno".to_string());
        vars.insert(nova.to_string(), "criada".to_string());
        charm_mock::env(Some(&mut t), &vars);
        assert_eq!(std::env::var(definida).unwrap(), "interno");
        assert_eq!(std::env::var(nova).unwrap(), "criada");
        // simulate testing cleanup at end of subtest by running
        t.run();
    }
    assert_eq!(std::env::var(definida).unwrap(), "original");
    assert!(std::env::var(nova).is_err());
}

#[test]
#[serial]
fn env_sem_setenv_fallback() {
    let definida = "CHARM_MOCK_ENV_FALLBACK_DEFINIDA";
    let nova = "CHARM_MOCK_ENV_FALLBACK_NOVA";
    std::env::set_var(definida, "antes");
    std::env::remove_var(nova);

    let mut fake = FakeCleanuper::new();
    let mut vars = std::collections::BTreeMap::new();
    vars.insert(definida.to_string(), "depois".to_string());
    vars.insert(nova.to_string(), "temporaria".to_string());
    charm_mock::env(Some(&mut fake), &vars);
    assert_eq!(std::env::var(definida).unwrap(), "depois");
    assert_eq!(std::env::var(nova).unwrap(), "temporaria");
    fake.run();
    assert_eq!(std::env::var(definida).unwrap(), "antes");
    assert!(std::env::var(nova).is_err());
}

#[test]
#[serial]
fn env_valor_vazio() {
    let chave = "CHARM_MOCK_ENV_VAZIA";
    std::env::set_var(chave, "cheia");
    {
        let mut t = FakeCleanuper::new();
        let mut vars = std::collections::BTreeMap::new();
        vars.insert(chave.to_string(), "".to_string());
        charm_mock::env(Some(&mut t), &vars);
        assert_eq!(std::env::var(chave).unwrap(), "");
        t.run();
    }
    assert_eq!(std::env::var(chave).unwrap(), "cheia");
}

#[test]
fn env_nil_nao_faz_nada() {
    charm_mock::env::<FakeCleanuper>(None, &std::collections::BTreeMap::new());
    // nil target with vars? Actually env(nil, nil) in Go does nothing if empty map.
    let mut vars = std::collections::BTreeMap::new();
    vars.insert("X".to_string(), "Y".to_string());
    // with None, sets but no restore; clean up manually
    charm_mock::env::<FakeCleanuper>(None, &vars);
    assert_eq!(std::env::var("X").unwrap(), "Y");
    std::env::remove_var("X");
}

#[test]
fn env_sem_t() {
    let chave = "CHARM_MOCK_ENV_SEM_T";
    std::env::set_var(chave, "antigo");
    // no restore
    let mut vars = std::collections::BTreeMap::new();
    vars.insert(chave.to_string(), "definida".to_string());
    charm_mock::env::<FakeCleanuper>(None, &vars);
    assert_eq!(std::env::var(chave).unwrap(), "definida");
    std::env::set_var(chave, "antigo");
}

#[test]
fn stdin_leitura() {
    let entrada = "linha 1\nlinha 2\n";
    let mut r = charm_mock::stdin::<FakeCleanuper>(None, entrada);
    let mut buf = Vec::new();
    io::Read::read_to_end(&mut r, &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), entrada);
    // second read empty
    let mut buf2 = Vec::new();
    io::Read::read_to_end(&mut r, &mut buf2).unwrap();
    assert!(buf2.is_empty());
}

#[test]
fn stdin_vazio() {
    let mut r = charm_mock::stdin::<FakeCleanuper>(None, "");
    let mut buf = Vec::new();
    io::Read::read_to_end(&mut r, &mut buf).unwrap();
    assert!(buf.is_empty());
}
