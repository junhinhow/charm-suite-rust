//! Tipo de cor canonico da Charm Suite Rust.
//!
//! O backend de styling ([`lipgloss`]) representa cores como strings
//! (`"#ff0000"`, `"63"`, `"5"`), enquanto as operacoes de cor (escurecer,
//! clarear, misturar) precisam de componentes numericos. [`Rgb`] faz essa
//! ponte e e o tipo de cor expublicado por toda a suite.

use crate::error::Error;

/// Uma cor RGBA de 8 bits por canal.
///
/// Equivale a `color.NRGBA` do Go original.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    /// Componente vermelho (0-255).
    pub r: u8,
    /// Componente verde (0-255).
    pub g: u8,
    /// Componente azul (0-255).
    pub b: u8,
    /// Componente alfa (0-255).
    pub a: u8,
}

impl Rgb {
    /// Cria uma cor a partir dos componentes de 8 bits.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Cria uma cor opaca a partir dos componentes de 8 bits.
    #[allow(clippy::self_named_constructors)]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Cria uma cor opaca preta (`#000000`).
    pub const fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    /// Cria uma cor opaca branca (`#ffffff`).
    pub const fn white() -> Self {
        Self::rgb(255, 255, 255)
    }

    /// Interpreta uma string hexadecimal (`#rrggbb` ou `rrggbb`).
    ///
    /// Retorna [`Error::InvalidColor`] para entradas malformadas.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::Rgb;
    ///
    /// assert_eq!(Rgb::from_hex("#282828").unwrap().to_hex(), "#282828");
    /// assert_eq!(Rgb::from_hex("282828").unwrap().to_hex(), "#282828");
    /// assert!(Rgb::from_hex("#28").is_err());
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::InvalidColor(hex.to_owned()));
        }
        let r =
            u8::from_str_radix(&hex[0..2], 16).map_err(|_| Error::InvalidColor(hex.to_owned()))?;
        let g =
            u8::from_str_radix(&hex[2..4], 16).map_err(|_| Error::InvalidColor(hex.to_owned()))?;
        let b =
            u8::from_str_radix(&hex[4..6], 16).map_err(|_| Error::InvalidColor(hex.to_owned()))?;
        Ok(Self::rgb(r, g, b))
    }

    /// Interpreta uma string hexadecimal, devolvendo preto opaco quando invalida.
    ///
    /// Equivale ao `FromHex` do Go original, que nunca falha e retorna
    /// `color.NRGBA{A: 255}` (preto opaco) para entradas invalidas.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::Rgb;
    ///
    /// assert_eq!(Rgb::from_hex_or_black("zzz"), Rgb::black());
    /// ```
    pub fn from_hex_or_black(hex: &str) -> Self {
        Self::from_hex(hex).unwrap_or_else(|_| Self::black())
    }

    /// Serializa a cor em `#rrggbb`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::Rgb;
    ///
    /// assert_eq!(Rgb::rgb(0xfa, 0xbd, 0x2f).to_hex(), "#fabd2f");
    /// ```
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Converte uma cor do backend [`lipgloss`] para [`Rgb`].
    ///
    /// Cores em formato ANSI (`"5"`) ou ANSI-256 (`"63"`) sao convertidas para
    /// RGB usando a tabela padrao do terminal.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::Rgb;
    ///
    /// let c = Rgb::from_lipgloss(&lipgloss::Color("#282828".to_string()));
    /// assert_eq!(c.to_hex(), "#282828");
    /// ```
    pub fn from_lipgloss(color: &lipgloss::Color) -> Self {
        let (r, g, b, a) = color.rgba16();
        Self::new(
            (r >> 8) as u8,
            (g >> 8) as u8,
            (b >> 8) as u8,
            (a >> 8) as u8,
        )
    }

    /// Converte uma cor do backend [`lipgloss`] para [`Rgb`], a partir de uma
    /// string (`"#ff0000"`, `"63"`, `"5"`).
    pub fn from_lipgloss_str(s: &str) -> Self {
        Self::from_lipgloss(&lipgloss::Color(s.to_owned()))
    }

    /// Converte a cor para o tipo do backend [`lipgloss`].
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::Rgb;
    ///
    /// let c = Rgb::rgb(0x28, 0x28, 0x28).to_lipgloss();
    /// assert_eq!(c.0, "#282828");
    /// ```
    pub fn to_lipgloss(self) -> lipgloss::Color {
        lipgloss::Color::from_rgba(self.r, self.g, self.b, self.a)
    }

    /// Converte a cor para [`palette::Srgb`] (espaco sRGB nao-linear).
    pub fn to_srgb(self) -> palette::Srgb {
        palette::Srgb::new(
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        )
    }

    /// Cria uma cor a partir de [`palette::Srgb`], fixando o alfa em opaco.
    pub fn from_srgb(c: palette::Srgb) -> Self {
        Self::rgb(
            (c.red * 255.0).round() as u8,
            (c.green * 255.0).round() as u8,
            (c.blue * 255.0).round() as u8,
        )
    }
}

impl Default for Rgb {
    fn default() -> Self {
        Self::black()
    }
}

impl std::fmt::Display for Rgb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let c = Rgb::from_hex("#282828").unwrap();
        assert_eq!(c, Rgb::rgb(0x28, 0x28, 0x28));
        assert_eq!(c.to_hex(), "#282828");
    }

    #[test]
    fn hex_without_hash() {
        assert_eq!(Rgb::from_hex("fabd2f").unwrap().to_hex(), "#fabd2f");
    }

    #[test]
    fn invalid_hex_falls_back_to_black() {
        assert_eq!(Rgb::from_hex_or_black("nope"), Rgb::black());
        assert_eq!(Rgb::from_hex_or_black("#ff00"), Rgb::black());
        assert!(Rgb::from_hex("#ff00").is_err());
    }

    #[test]
    fn lipgloss_bridge_hex() {
        let c = Rgb::from_lipgloss_str("#282828");
        assert_eq!(c.to_hex(), "#282828");
        assert_eq!(c.to_lipgloss().0, "#282828");
    }

    #[test]
    fn lipgloss_bridge_ansi256() {
        // "63" e um indice ANSI-256 que precisa ser convertido para RGB.
        let c = Rgb::from_lipgloss_str("63");
        assert_eq!(c.a, 255);
        // Nao pode ser preto puro (indice invalido seria 0).
        assert_ne!(c, Rgb::black());
    }

    #[test]
    fn srgb_bridge() {
        let c = Rgb::rgb(255, 0, 0);
        assert_eq!(Rgb::from_srgb(c.to_srgb()), c);
    }
}
