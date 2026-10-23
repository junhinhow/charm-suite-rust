#![allow(clippy::all, clippy::pedantic, unused_mut, unused_variables, dead_code)]

use super::*;
use charmed_bubbletea::{KeyMsg, KeyType, Message, WindowSizeMsg};

fn novo_layout(dir: Direction, largura: i32, altura: i32, panes: &[Pane]) -> Layout {
    let mut l = Layout::new(dir);
    l.set_size(largura, altura);
    for p in panes {
        l.add(p.clone());
    }
    l
}

fn somar(tamanhos: &[i32], visiveis: i32) -> i32 {
    let mut soma = 0;
    for i in 0..visiveis as usize {
        if i < tamanhos.len() {
            soma += tamanhos[i];
        }
    }
    soma
}

#[test]
fn soma_exata_horizontal() {
    let larguras = [80, 81, 40, 7];
    let quantidades = [2, 3, 5];
    for &largura in &larguras {
        for &quantidade in &quantidades {
            let mut l = Layout::new(Direction::Horizontal);
            for i in 0..quantidade {
                l.add(Pane {
                    id: format!("p{}", i),
                    ..Default::default()
                });
            }
            l.set_size(largura, 10);
            let tamanhos = l.sizes();
            assert_eq!(tamanhos.len(), quantidade);
            let visiveis = l.visible();
            let soma = somar(&tamanhos, visiveis);
            if visiveis > 0 {
                assert_eq!(soma + visiveis - 1, largura);
            }
            for i in 0..visiveis as usize {
                assert!(tamanhos[i] >= 3);
            }
            for i in visiveis as usize..quantidade {
                assert_eq!(tamanhos[i], 0);
            }
        }
    }
}

#[test]
fn soma_exata_vertical() {
    let alturas = [80, 81, 40, 7];
    for &altura in &alturas {
        for &quantidade in &[2, 3, 5] {
            let mut l = Layout::new(Direction::Vertical);
            for i in 0..quantidade {
                l.add(Pane {
                    id: format!("p{}", i),
                    ..Default::default()
                });
            }
            l.set_size(40, altura);
            let tamanhos = l.sizes();
            let visiveis = l.visible();
            let soma = somar(&tamanhos, visiveis);
            if visiveis > 0 {
                assert_eq!(soma + visiveis - 1, altura);
            }
        }
    }
}

#[test]
fn min_size_respeitado() {
    let mut l = novo_layout(
        Direction::Horizontal,
        30,
        8,
        &[
            Pane {
                id: "a".into(),
                min_size: 10,
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                min_size: 5,
                ..Default::default()
            },
        ],
    );
    let tamanhos = l.sizes();
    assert_eq!(l.visible(), 2);
    assert!(tamanhos[0] >= 10);
    assert!(tamanhos[1] >= 5);
    assert_eq!(tamanhos[0] + tamanhos[1] + 1, 30);
    l.set_size(12, 8);
    assert_eq!(l.visible(), 1);
    let tamanhos = l.sizes();
    assert!(tamanhos[0] >= 10);
    assert_eq!(tamanhos[1], 0);
}

#[test]
fn min_size_padrao() {
    let mut l = novo_layout(
        Direction::Horizontal,
        80,
        10,
        &[Pane {
            id: "a".into(),
            min_size: -5,
            ..Default::default()
        }],
    );
    assert_eq!(l.sizes()[0], 80);
    let (p, ok) = l.pane("a");
    assert!(ok);
    assert_eq!(p.min_size, 3);
}

#[test]
fn resize_para_cima_e_para_baixo() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    let iniciais = l.sizes();
    assert_eq!(iniciais, vec![20, 19, 19]);
    l.focus(0);
    l.resize(10);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0], 30);
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 60);
    l.resize(-100);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0], 3);
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 60);
    l.focus(1);
    l.resize(1000);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 60);
    assert!(tamanhos[0] >= 3 && tamanhos[2] >= 3);
}

#[test]
fn resize_to() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(0);
    let atual = l.sizes()[0];
    l.resize_to(atual);
    assert_eq!(l.sizes()[0], atual);
    l.resize_to(30);
    assert_eq!(l.sizes()[0], 30);
    l.resize_to(0);
    assert_eq!(l.sizes()[0], 3);
    l.resize_to(9999);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 60);
    assert!(tamanhos[1] >= 3 && tamanhos[2] >= 3);
}

#[test]
fn fixed_nao_cede() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "fixo".into(),
                fixed: true,
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    let iniciais = l.sizes();
    assert_eq!(iniciais[0], 20);
    l.focus(1);
    l.resize(5);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0], iniciais[0]);
    assert_eq!(tamanhos[1], 24);
    l.set_size(80, 10);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0], iniciais[0]);
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 80);
    l.focus(0);
    l.resize_to(30);
    assert_eq!(l.sizes()[0], 30);
    l.resize_to(10);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0], 10);
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 80);
}

#[test]
fn fixed_todos_mantem_soma() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[
            Pane {
                id: "a".into(),
                fixed: true,
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                fixed: true,
                ..Default::default()
            },
        ],
    );
    l.focus(0);
    l.resize(-5);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos[0], 15);
    assert_eq!(tamanhos[0] + tamanhos[1] + 1, 40);
    l.resize(5);
    assert_eq!(l.sizes()[0], 15);
}

#[test]
fn foco_circular_e_clampado() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(999);
    assert_eq!(l.focused(), 2);
    l.focus(-7);
    assert_eq!(l.focused(), 0);
    l.focus_next();
    l.focus_next();
    l.focus_next();
    assert_eq!(l.focused(), 0);
    l.focus_prev();
    assert_eq!(l.focused(), 2);
    let mut vazio = Layout::new(Direction::Horizontal);
    assert_eq!(vazio.focused(), -1);
    vazio.focus_next();
    vazio.focus_prev();
    vazio.focus(10);
    assert_eq!(vazio.focused(), -1);
}

#[test]
fn add_remove_e_pane() {
    let mut l = Layout::new(Direction::Horizontal);
    l.set_size(40, 8);
    l.add(Pane {
        id: "a".into(),
        title: "A".into(),
        content: "um".into(),
        ..Default::default()
    });
    l.add(Pane {
        id: "b".into(),
        title: "B".into(),
        content: "dois".into(),
        ..Default::default()
    });
    assert_eq!(l.ids(), vec!["a", "b"]);
    l.set_content("a", "novo");
    l.set_title("a", "Novo");
    let (p, ok) = l.pane("a");
    assert!(ok);
    assert_eq!(p.content, "novo");
    assert_eq!(p.title, "Novo");
    let (_, ok2) = l.pane("inexistente");
    assert!(!ok2);
    l.set_content("inexistente", "x");
    l.set_title("inexistente", "x");
    assert!(l.remove("a"));
    assert!(!l.remove("a"));
    assert_eq!(l.ids(), vec!["b"]);
    l.add(Pane {
        id: "c".into(),
        ..Default::default()
    });
    l.idrop("c");
    assert_eq!(l.ids(), vec!["b"]);
}

#[test]
fn add_duplicado_substitui() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[
            Pane {
                id: "a".into(),
                content: "antigo".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                content: "b".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(1);
    l.add(Pane {
        id: "a".into(),
        title: "novo".into(),
        content: "substituto".into(),
        ..Default::default()
    });
    assert_eq!(l.ids(), vec!["a", "b"]);
    let (p, _) = l.pane("a");
    assert_eq!(p.content, "substituto");
    assert_eq!(p.title, "novo");
    assert_eq!(l.focused(), 1);
    assert_eq!(l.sizes().len(), 2);
}

#[test]
fn remove_ajusta_foco() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(2);
    l.remove("c");
    assert_eq!(l.focused(), 1);
    l.focus(1);
    l.remove("a");
    assert_eq!(l.focused(), 0);
}

#[test]
fn reset() {
    let mut l = novo_layout(
        Direction::Horizontal,
        80,
        10,
        &[
            Pane {
                id: "a".into(),
                min_size: 5,
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(0);
    l.resize_to(60);
    l.reset();
    let tamanhos = l.sizes();
    assert!(tamanhos[0] >= 5);
    assert_eq!(tamanhos[1], tamanhos[2]);
    assert_eq!(tamanhos[0] + tamanhos[1] + tamanhos[2] + 2, 80);
    l.add(Pane {
        id: "d".into(),
        ..Default::default()
    });
    let tamanhos = l.sizes();
    assert_eq!(tamanhos.len(), 4);
    assert_eq!(
        tamanhos[0] + tamanhos[1] + tamanhos[2] + tamanhos[3] + 3,
        80
    );
}

#[test]
fn reset_com_minimos_grandes() {
    let mut l = novo_layout(
        Direction::Horizontal,
        20,
        8,
        &[
            Pane {
                id: "a".into(),
                min_size: 15,
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
        ],
    );
    l.reset();
    let tamanhos = l.sizes();
    assert!(tamanhos[0] >= 15);
    assert!(tamanhos[1] >= 3);
    assert_eq!(tamanhos[0] + tamanhos[1] + 1, 20);
}

#[test]
fn reset_sem_espaco() {
    let mut l = novo_layout(
        Direction::Horizontal,
        20,
        8,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
        ],
    );
    l.set_size(2, 8);
    l.reset();
    for t in l.sizes() {
        assert_eq!(t, 0);
    }
    assert_eq!(l.view(), "");
}

#[test]
fn degradacao() {
    let mut l = Layout::new(Direction::Horizontal);
    for i in 0..5 {
        l.add(Pane {
            id: format!("p{}", i),
            ..Default::default()
        });
    }
    l.set_size(7, 10);
    assert_eq!(l.visible(), 2);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos.len(), 5);
    assert_eq!(tamanhos[0], 3);
    assert_eq!(tamanhos[1], 3);
    for i in 2..5 {
        assert_eq!(tamanhos[i], 0);
    }
    assert!(l.view().contains("+3"));
    l.set_size(80, 10);
    assert_eq!(l.visible(), 5);
    assert_eq!(somar(&l.sizes(), 5) + 4, 80);
}

#[test]
fn set_size_negativo() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
        ],
    );
    l.set_size(-10, -20);
    assert_eq!(l.size(), (0, 0));
    assert_eq!(l.view(), "");
}

#[test]
fn ids_sao_copia() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
        ],
    );
    let mut ids = l.ids();
    ids[0] = "alterado".to_string();
    assert_eq!(l.ids()[0], "a");
}

#[test]
fn sizes_sao_copia() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
        ],
    );
    let mut tamanhos = l.sizes();
    tamanhos[0] = 999;
    assert_ne!(l.sizes()[0], 999);
}

#[test]
fn resize_sem_foco_visivel() {
    let mut l = Layout::new(Direction::Horizontal);
    for i in 0..5 {
        l.add(Pane {
            id: format!("p{}", i),
            ..Default::default()
        });
    }
    l.set_size(7, 10);
    l.focus(4);
    let antes = l.sizes();
    l.resize(10);
    let depois = l.sizes();
    assert_eq!(antes, depois);
}

#[test]
fn resize_painel_unico() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[Pane {
            id: "solo".into(),
            content: "x".into(),
            ..Default::default()
        }],
    );
    l.resize(-5);
    l.resize(5);
    l.resize_to(10);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos.len(), 1);
    assert_eq!(tamanhos[0], 40);
    assert!(!l.view().is_empty());
}

#[test]
fn resize_unico_visivel() {
    let mut l = Layout::new(Direction::Horizontal);
    for i in 0..5 {
        l.add(Pane {
            id: format!("p{}", i),
            ..Default::default()
        });
    }
    l.set_size(7, 10);
    l.focus(0);
    let antes = l.sizes();
    l.resize(-1);
    let depois = l.sizes();
    assert_eq!(antes, depois);
    assert_eq!(somar(&depois, l.visible()) + l.visible() - 1, 7);
}

#[test]
fn update_navegacao() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(0);
    let cmd = l.update(Message::new(KeyMsg::from_type(KeyType::Tab)));
    assert!(cmd.is_none());
    assert_eq!(l.focused(), 1);
    l.update(Message::new(KeyMsg::from_type(KeyType::ShiftTab)));
    assert_eq!(l.focused(), 0);
    l.update(Message::new(KeyMsg::from_type(KeyType::ShiftTab)));
    assert_eq!(l.focused(), 2);
}

#[test]
fn update_resize_horizontal() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(0);
    let casos: Vec<(&str, KeyMsg, i32)> = vec![
        ("alt+right", KeyMsg::from_type(KeyType::Right).with_alt(), 1),
        ("alt+left", KeyMsg::from_type(KeyType::Left).with_alt(), -1),
        ("+", KeyMsg::from_char('+'), 1),
        ("-", KeyMsg::from_char('-'), -1),
        ("=", KeyMsg::from_char('='), 1),
        ("_", KeyMsg::from_char('_'), -1),
    ];
    for (_nome, tecla, delta) in casos {
        let antes = l.sizes();
        l.update(Message::new(tecla));
        let depois = l.sizes();
        assert_eq!(depois[0] - antes[0], delta);
        assert_eq!(depois[0] + depois[1] + depois[2] + 2, 60);
    }
}

#[test]
fn update_resize_vertical() {
    let mut l = novo_layout(
        Direction::Vertical,
        40,
        60,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(0);
    let antes = l.sizes();
    l.update(Message::new(KeyMsg::from_type(KeyType::Right).with_alt()));
    assert_eq!(l.sizes()[0], antes[0]);
    l.update(Message::new(KeyMsg::from_type(KeyType::Down).with_alt()));
    assert_eq!(l.sizes()[0], antes[0] + 1);
    l.update(Message::new(KeyMsg::from_type(KeyType::Up).with_alt()));
    assert_eq!(l.sizes()[0], antes[0]);
}

#[test]
fn update_window_size() {
    let mut l = Layout::new(Direction::Horizontal);
    l.add(Pane {
        id: "a".into(),
        content: "um".into(),
        ..Default::default()
    });
    l.add(Pane {
        id: "b".into(),
        content: "dois".into(),
        ..Default::default()
    });
    l.update(Message::new(WindowSizeMsg {
        width: 33,
        height: 7,
    }));
    assert_eq!(l.size(), (33, 7));
    let view = l.view();
    let linhas: Vec<&str> = view.split('\n').collect();
    assert_eq!(linhas.len(), 7);
    for linha in linhas {
        assert_eq!(charm_core::display_width(linha), 33);
    }
}

#[test]
fn update_tecla_desconhecida() {
    let mut l = novo_layout(
        Direction::Horizontal,
        60,
        10,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
        ],
    );
    l.focus(1);
    let antes = l.sizes();
    l.update(Message::new(KeyMsg::from_char('x')));
    assert_eq!(l.focused(), 1);
    assert_eq!(l.sizes()[1], antes[1]);
}

#[test]
fn init_e_view_model() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        6,
        &[
            Pane {
                id: "a".into(),
                title: "A".into(),
                content: "um".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                title: "B".into(),
                content: "dois".into(),
                ..Default::default()
            },
        ],
    );
    assert!(l.init().is_none());
    let vm = l.view_model();
    assert_eq!(vm.content, l.view());
}

#[test]
fn view_vazio() {
    let mut l = Layout::new(Direction::Horizontal);
    l.set_size(40, 10);
    assert_eq!(l.view(), "");
}

#[test]
fn view_um_painel() {
    let mut l = novo_layout(
        Direction::Horizontal,
        20,
        3,
        &[Pane {
            id: "solo".into(),
            title: "Titulo".into(),
            content: "ola".into(),
            ..Default::default()
        }],
    );
    let view = l.view();
    assert_eq!(view.split('\n').count(), 3);
    assert!(view.starts_with("ola"));
    assert!(!view.contains('│'));
    assert!(!view.contains('▌'));
}

#[test]
fn view_dois_paineis() {
    let mut l = novo_layout(
        Direction::Horizontal,
        20,
        4,
        &[
            Pane {
                id: "a".into(),
                title: "Lado".into(),
                content: "esq".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                title: "Editor".into(),
                content: "dir".into(),
                ..Default::default()
            },
        ],
    );
    let view = l.view();
    let linhas: Vec<&str> = view.split('\n').collect();
    assert_eq!(linhas.len(), 4);
    let tamanhos = l.sizes();
    let divisor_col = charm_core::display_width(linhas[0].split('│').next().unwrap());
    assert_eq!(divisor_col, tamanhos[0] as usize);
    assert!(linhas[0].starts_with("▌ Lado"));
    assert!(linhas[0].contains(" Editor"));
    assert_eq!(linhas[0].matches('│').count(), 1);
    l.focus_next();
    let view = l.view();
    let linha0 = view.split('\n').next().unwrap();
    assert_eq!(linha0.matches("▌").count(), 1);
    assert!(linha0.contains("▌ Editor"));
}

#[test]
fn view_vertical() {
    let mut l = novo_layout(
        Direction::Vertical,
        20,
        10,
        &[
            Pane {
                id: "a".into(),
                title: "Topo".into(),
                content: "linha".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                title: "Base".into(),
                content: "fundo".into(),
                ..Default::default()
            },
        ],
    );
    let view = l.view();
    let linhas: Vec<&str> = view.split('\n').collect();
    assert_eq!(linhas.len(), 10);
    let tamanhos = l.sizes();
    assert_eq!(tamanhos, vec![5, 4]);
    let divisor = "─".repeat(20);
    assert_eq!(linhas[tamanhos[0] as usize], divisor);
    assert_eq!(view.matches(&divisor).count(), 1);
    assert!(linhas[0].starts_with("▌ Topo"));
    assert!(linhas[tamanhos[0] as usize + 1].starts_with("  Base"));
}

#[test]
fn view_titulo_multibyte() {
    let mut l = novo_layout(
        Direction::Horizontal,
        20,
        4,
        &[
            Pane {
                id: "a".into(),
                title: "Ação".into(),
                content: "conteudo".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                title: "Café".into(),
                content: "texto".into(),
                ..Default::default()
            },
        ],
    );
    let view = l.view();
    assert!(view.contains("Ação"));
    assert!(std::str::from_utf8(view.as_bytes()).is_ok());
    l.set_size(9, 4);
    let view = l.view();
    assert_eq!(view.split('\n').count(), 4);
    for line in view.split('\n') {
        assert_eq!(charm_core::display_width(line), 9);
    }
    assert!(std::str::from_utf8(view.as_bytes()).is_ok());
}

#[test]
fn view_largura_zero() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[
            Pane {
                id: "a".into(),
                content: "x".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                content: "y".into(),
                ..Default::default()
            },
        ],
    );
    l.set_size(0, 8);
    assert_eq!(l.view(), "");
    l.set_size(40, 0);
    assert_eq!(l.view(), "");
    l.set_size(0, 0);
    assert_eq!(l.view(), "");
}

#[test]
fn view_abaixo_do_minimo() {
    let mut l = novo_layout(
        Direction::Horizontal,
        40,
        8,
        &[Pane {
            id: "solo".into(),
            content: "x".into(),
            ..Default::default()
        }],
    );
    l.set_size(2, 8);
    assert_eq!(l.view(), "");
    l.set_size(3, 8);
    assert!(!l.view().is_empty());
}

#[test]
fn view_divisores_exatos() {
    let mut l = novo_layout(
        Direction::Horizontal,
        31,
        6,
        &[
            Pane {
                id: "a".into(),
                ..Default::default()
            },
            Pane {
                id: "b".into(),
                ..Default::default()
            },
            Pane {
                id: "c".into(),
                ..Default::default()
            },
            Pane {
                id: "d".into(),
                ..Default::default()
            },
        ],
    );
    let tamanhos = l.sizes();
    let view = l.view();
    for linha in view.split('\n') {
        let partes: Vec<&str> = linha.split('│').collect();
        assert_eq!(partes.len(), 4);
        for (i, parte) in partes.iter().enumerate() {
            assert_eq!(charm_core::display_width(parte), tamanhos[i] as usize);
        }
    }
}

#[test]
fn example_output() {
    let mut l = Layout::new(Direction::Horizontal);
    l.set_size(60, 8);
    l.add(Pane {
        id: "sidebar".into(),
        title: "Arquivos".into(),
        content: "main.go\nREADME.md".into(),
        min_size: 12,
        fixed: true,
    });
    l.add(Pane {
        id: "editor".into(),
        title: "Editor".into(),
        content: "package main\n\nfunc main() {}".into(),
        min_size: 0,
        fixed: false,
    });
    l.add(Pane {
        id: "terminal".into(),
        title: "Terminal".into(),
        content: "$ go test ./...".into(),
        min_size: 0,
        fixed: false,
    });
    l.focus(1);
    l.resize(10);
    assert_eq!(
        charm_core::display_width(&l.view().split('\n').next().unwrap_or("")),
        60
    ); // rough
    assert_eq!(l.sizes(), vec![20, 29, 9]);
    l.update(Message::new(KeyMsg::from_type(KeyType::Right).with_alt()));
    assert_eq!(l.focused(), 1);
    assert_eq!(l.sizes()[1], 30);
}
