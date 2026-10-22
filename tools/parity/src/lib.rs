//! Harness de paridade entre a implementacao Go original e o port Rust.
//!
//! Os *golden files* sao gerados executando a implementacao Go e ficam em
//! `tests/golden/<caso>.txt` dentro de cada crate. Este crate oferece:
//!
//! - [`assert_golden`]: compara a saida Rust com o golden file, ignorando
//!   codigos ANSI (o `lipgloss` Go nao emite cores quando stdout nao e um
//!   terminal, enquanto o port Rust sempre emite);
//! - [`assert_golden_inline`]: compara com um valor esperado embutido no
//!   proprio teste;
//! - [`update_goldens`]: indica se a variavel de ambiente `CHARM_PARITY_UPDATE`
//!   pede a regeneracao automatica dos goldens.
//!
//! ```ignore
//! use charm_parity::assert_golden;
//!
//! assert_golden("nav_tabs", &rendered);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::path::{Path, PathBuf};

/// Variavel de ambiente que habilita a regeneracao automatica dos goldens.
pub const UPDATE_ENV: &str = "CHARM_PARITY_UPDATE";

/// Retorna `true` quando a regeneracao automatica dos goldens esta habilitada.
pub fn update_goldens() -> bool {
    std::env::var(UPDATE_ENV).is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Caminho absoluto do arquivo golden de um caso.
///
/// Resolvido a partir do diretorio de trabalho do processo de teste, que o
/// Cargo define como a raiz do pacote em teste.
pub fn golden_path(case: &str) -> PathBuf {
    Path::new("tests")
        .join("golden")
        .join(format!("{case}.txt"))
}

/// Normaliza uma saida para comparacao: remove ANSI, normaliza quebras de
/// linha e remove espacos no fim de cada linha.
pub fn normalize(s: &str) -> String {
    let stripped = charm_core::strip_ansi(s);
    stripped
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

/// Compara a saida atual com o golden file do caso.
///
/// Se `CHARM_PARITY_UPDATE=1` estiver definido, o golden file e reescrito com a
/// saida atual e a funcao retorna sem falhar.
///
/// # Panics
///
/// Panica se a saida divergir do golden file.
pub fn assert_golden(case: &str, actual: &str) {
    let path = golden_path(case);
    if update_goldens() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, actual).expect("falha ao escrever golden file");
        eprintln!("[parity] golden atualizado: {}", path.display());
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "golden file ausente: {} ({e}).\nGere-o com CHARM_PARITY_UPDATE=1 cargo test -p <crate>",
            path.display()
        )
    });

    let expected_norm = normalize(&expected);
    let actual_norm = normalize(actual);
    assert!(
        expected_norm == actual_norm,
        "divergencia de paridade no caso `{case}`:\n--- esperado (Go) ---\n{expected_norm}\n--- obtido (Rust) ---\n{actual_norm}\n--- fim ---"
    );
}

/// Compara a saida atual com um valor esperado embutido no teste.
///
/// # Panics
///
/// Panica se a saida divergir do valor esperado.
pub fn assert_golden_inline(case: &str, expected: &str, actual: &str) {
    let expected_norm = normalize(expected);
    let actual_norm = normalize(actual);
    assert!(
        expected_norm == actual_norm,
        "divergencia de paridade no caso `{case}`:\n--- esperado (Go) ---\n{expected_norm}\n--- obtido (Rust) ---\n{actual_norm}\n--- fim ---"
    );
}
