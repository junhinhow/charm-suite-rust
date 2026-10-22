//! Erro padrao da Charm Suite Rust.

use thiserror::Error;

/// Erro padronizado utilizado pelas bibliotecas da suite.
///
/// Cada crate pode estender este tipo com variantes proprias (via
/// `#[from]` do thiserror) ou simplesmente definir seu proprio enum de
/// erro; este tipo cobre os casos transversais (cor invalida, I/O).
#[derive(Debug, Error)]
pub enum Error {
    /// Valor de cor invalido (hexadecimal malformado, por exemplo).
    #[error("invalid color value: {0}")]
    InvalidColor(String),

    /// Erro de entrada/saida.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Erro generico com mensagem.
    #[error("{0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Other(value.to_owned())
    }
}
