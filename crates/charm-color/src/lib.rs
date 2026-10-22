//! Utilitarios de manipulacao de cores para interfaces de terminal.
//!
//! As cores sao manipuladas no modelo de alfa reto (straight alpha, NRGBA):
//! o canal alfa nunca e aplicado aos canais RGB. Isso evita que operacoes
//! como escurecer, clarear e misturar escurecam silenciosamente as cores com
//! transparencia.
//!
//! # Examples
//!
//! ```
//! use charm_color::{darken, Rgb};
//!
//! let red = Rgb::rgb(255, 0, 0);
//! assert_eq!(darken(red, 0.5).to_hex(), "#800000");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::many_single_char_names,
    clippy::float_cmp,
    clippy::manual_midpoint
)]

use charm_core::Error;
pub use charm_core::Rgb;

/// Escurece a cor pelo fator `amount` (0.0 a 1.0).
///
/// A cor e convertida para alfa reto antes da operacao, portanto o canal alfa
/// e preservado e `darken(c, 0)` devolve a propria cor.
pub fn darken(c: Rgb, amount: f64) -> Rgb {
    let n = c;
    let factor = clamp(1.0 - amount);
    Rgb::new(
        para_uint8(f64::from(n.r) * factor),
        para_uint8(f64::from(n.g) * factor),
        para_uint8(f64::from(n.b) * factor),
        n.a,
    )
}

/// Clareia a cor pelo fator `amount` (0.0 a 1.0).
pub fn lighten(c: Rgb, amount: f64) -> Rgb {
    let n = c;
    let amt = clamp(amount);
    Rgb::new(
        para_uint8(f64::from(n.r) + (255.0 - f64::from(n.r)) * amt),
        para_uint8(f64::from(n.g) + (255.0 - f64::from(n.g)) * amt),
        para_uint8(f64::from(n.b) + (255.0 - f64::from(n.b)) * amt),
        n.a,
    )
}

/// Interpola entre `c1` e `c2` pelo fator `t` (0.0 = c1, 1.0 = c2).
pub fn blend(c1: Rgb, c2: Rgb, t: f64) -> Rgb {
    let t = clamp(t);
    Rgb::new(
        para_uint8(lerp(f64::from(c1.r), f64::from(c2.r), t)),
        para_uint8(lerp(f64::from(c1.g), f64::from(c2.g), t)),
        para_uint8(lerp(f64::from(c1.b), f64::from(c2.b), t)),
        para_uint8(lerp(f64::from(c1.a), f64::from(c2.a), t)),
    )
}

/// Retorna preto ou branco dependendo da luminancia da cor.
pub fn contrast(c: Rgb) -> Rgb {
    let n = c;
    let lum = 0.2126 * srgb_to_linear(f64::from(n.r) / 255.0)
        + 0.7152 * srgb_to_linear(f64::from(n.g) / 255.0)
        + 0.0722 * srgb_to_linear(f64::from(n.b) / 255.0);
    if lum > 0.179 {
        Rgb::rgb(0, 0, 0)
    } else {
        Rgb::rgb(255, 255, 255)
    }
}

/// Retorna a cor complementar (matiz deslocado em 180 graus), preservando o alfa.
pub fn complementary(c: Rgb) -> Rgb {
    let n = c;
    let (h, s, l) = rgb_to_hsl(n);
    hsl_to_nrgba((h + 180.0).rem_euclid(360.0), s, l, n.a)
}

/// Inverte os canais RGB da cor (255 - canal), preservando o alfa.
pub fn inverse(c: Rgb) -> Rgb {
    let n = c;
    Rgb::new(255 - n.r, 255 - n.g, 255 - n.b, n.a)
}

/// Gera uma paleta harmoniosa de `n` cores a partir de uma cor base,
/// rotacionando o hue uniformemente. O alfa da cor base e preservado.
pub fn palette(base: Rgb, n: usize) -> Vec<Rgb> {
    if n == 0 {
        return Vec::new();
    }
    let nb = base;
    let (h, s, l) = rgb_to_hsl(nb);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let hue = (h + (i as f64) * (360.0 / (n as f64))).rem_euclid(360.0);
        out.push(hsl_to_nrgba(hue, s, l, nb.a));
    }
    out
}

/// Converte uma cor para `#rrggbb` (alfa ignorado).
pub fn to_hex(c: Rgb) -> String {
    let n = c;
    format!("#{:02x}{:02x}{:02x}", n.r, n.g, n.b)
}

/// Converte uma cor para `#rrggbbaa`, incluindo o alfa.
pub fn to_hex_a(c: Rgb) -> String {
    let n = c;
    format!("#{:02x}{:02x}{:02x}{:02x}", n.r, n.g, n.b, n.a)
}

/// Converte uma string hexadecimal (`#rrggbb` ou `rrggbb`) para `Rgb`,
/// devolvendo preto opaco em caso de formato invalido.
pub fn from_hex(hex: &str) -> Rgb {
    from_hex_e(hex).unwrap_or_else(|_| Rgb::rgb(0, 0, 0))
}

/// Converte uma string hexadecimal para `Rgb`, validando o formato.
///
/// Aceita `#rrggbb`, `rrggbb`, `#rrggbbaa` e `rrggbbaa`. Devolve
/// [`charm_core::Error::InvalidHex`] quando a string nao e valida.
pub fn from_hex_e(hex: &str) -> Result<Rgb, Error> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 && s.len() != 8 {
        return Err(Error::InvalidHex(format!(
            "{hex:?} nao tem 6 nem 8 digitos hexadecimais"
        )));
    }
    let value = u64::from_str_radix(s, 16)
        .map_err(|_| Error::InvalidHex(format!("{hex:?} nao e um numero hexadecimal valido")))?;
    if s.len() == 6 {
        Ok(Rgb::new(
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
            255,
        ))
    } else {
        Ok(Rgb::new(
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ))
    }
}

/// Converte uma string hex/ANSI do lipgloss para `Rgb`.
pub fn from_lipgloss(s: &str) -> Rgb {
    Rgb::from_lipgloss_str(s)
}

// --- helpers ---

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn srgb_to_linear(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn rgb_to_hsl(c: Rgb) -> (f64, f64, f64) {
    let rf = f64::from(c.r) / 255.0;
    let gf = f64::from(c.g) / 255.0;
    let bf = f64::from(c.b) / 255.0;

    let max_canal = rf.max(gf.max(bf));
    let min_canal = rf.min(gf.min(bf));
    let l = (max_canal + min_canal) / 2.0;

    if max_canal == min_canal {
        return (0.0, 0.0, l);
    }

    let d = max_canal - min_canal;
    let s = if l > 0.5 {
        d / (2.0 - max_canal - min_canal)
    } else {
        d / (max_canal + min_canal)
    };

    let h = match max_canal {
        x if x == rf => {
            let mut h = (gf - bf) / d;
            if gf < bf {
                h += 6.0;
            }
            h
        }
        x if x == gf => (bf - rf) / d + 2.0,
        _ => (rf - gf) / d + 4.0,
    };
    let h = h * 60.0;
    (h, s, l)
}

fn hsl_to_nrgba(h: f64, s: f64, l: f64, a: u8) -> Rgb {
    if s == 0.0 {
        let v = para_uint8(l * 255.0);
        return Rgb::new(v, v, v, a);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let h_norm = h / 360.0;

    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);

    Rgb::new(
        para_uint8(r * 255.0),
        para_uint8(g * 255.0),
        para_uint8(b * 255.0),
        a,
    )
}

fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn para_uint8(v: f64) -> u8 {
    let r = v.round();
    if r <= 0.0 {
        0
    } else if r >= 255.0 {
        255
    } else {
        r as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn darken_reduces() {
        let c = Rgb::new(200, 100, 50, 255);
        let d = darken(c, 0.5);
        assert!(d.r <= 200 && d.g <= 100 && d.b <= 50);
    }

    #[test]
    fn lighten_increases() {
        let c = Rgb::new(100, 50, 25, 255);
        let l = lighten(c, 0.5);
        assert!(l.r >= 100 && l.g >= 50 && l.b >= 25);
    }

    #[test]
    fn blend_mid() {
        let black = Rgb::rgb(0, 0, 0);
        let white = Rgb::rgb(255, 255, 255);
        let mid = blend(black, white, 0.5);
        assert!(mid.r >= 120 && mid.r <= 135);
    }

    #[test]
    fn contrast_dark() {
        let white = Rgb::rgb(255, 255, 255);
        let c = contrast(white);
        assert_eq!(c, Rgb::rgb(0, 0, 0));
    }

    #[test]
    fn contrast_light() {
        let black = Rgb::rgb(0, 0, 0);
        let c = contrast(black);
        assert_eq!(c, Rgb::rgb(255, 255, 255));
    }

    #[test]
    fn complementary_red() {
        let c = Rgb::rgb(255, 0, 0);
        let comp = complementary(c);
        // matiz + 180
        assert!(comp.r < 128 && comp.g > 128 && comp.b > 128);
    }

    #[test]
    fn inverse_channels() {
        let inv = inverse(Rgb::new(0x40, 0x20, 0x80, 128));
        assert_eq!(inv, Rgb::new(0xbf, 0xdf, 0x7f, 128));
    }

    #[test]
    fn palette_size() {
        let p = palette(Rgb::rgb(255, 0, 0), 4);
        assert_eq!(p.len(), 4);
    }

    #[test]
    fn palette_zero() {
        let p = palette(Rgb::rgb(255, 0, 0), 0);
        assert!(p.is_empty(), "palette should be empty");
    }

    #[test]
    fn to_hex_basic() {
        assert_eq!(to_hex(Rgb::new(255, 128, 0, 255)), "#ff8000");
    }

    #[test]
    fn to_hex_a_basic() {
        assert_eq!(to_hex_a(Rgb::new(255, 0, 0, 128)), "#ff000080");
    }

    #[test]
    fn from_hex_roundtrip() {
        let c = from_hex("#ff8000");
        assert_eq!(c, Rgb::rgb(255, 128, 0));
    }

    #[test]
    fn from_hex_without_hash() {
        let c = from_hex("00ff00");
        assert_eq!(c, Rgb::rgb(0, 255, 0));
    }

    #[test]
    fn from_hex_invalid() {
        let c = from_hex("invalid");
        assert_eq!(c, Rgb::rgb(0, 0, 0));
    }

    #[test]
    fn from_hex_e_invalid() {
        assert!(from_hex_e("invalid").is_err());
        assert!(from_hex_e("").is_err());
        assert!(from_hex_e("#fff").is_err());
    }

    #[test]
    fn from_hex_e_with_alpha() {
        let c = from_hex_e("#ff000080").unwrap();
        assert_eq!(c, Rgb::new(255, 0, 0, 128));
    }

    #[test]
    fn from_hex_e_valid() {
        let c = from_hex_e("#ff8000").unwrap();
        assert_eq!(c, Rgb::new(255, 128, 0, 255));
    }

    #[test]
    fn darken_identity() {
        let c = Rgb::new(255, 0, 0, 128);
        let dark = darken(c, 0.0);
        assert_eq!(dark, c);
    }

    #[test]
    fn lighten_identity() {
        let c = Rgb::new(255, 0, 0, 128);
        let light = lighten(c, 0.0);
        assert_eq!(light, c);
    }

    #[test]
    fn darken_preserves_alpha() {
        let dark = darken(Rgb::new(255, 0, 0, 128), 0.5);
        assert_eq!(dark.a, 128);
        assert_eq!(dark.r, 128);
    }

    #[test]
    fn lighten_preserves_alpha() {
        let light = lighten(Rgb::new(100, 50, 25, 128), 0.5);
        assert_eq!(light.a, 128);
    }

    #[test]
    fn blend_preserves_alpha() {
        let a = Rgb::new(255, 0, 0, 128);
        let b = Rgb::new(0, 0, 255, 0);
        let mix = blend(a, b, 0.5);
        assert_eq!(mix.a, 64);
        assert_eq!(mix.r, 128);
    }

    #[test]
    fn rounding() {
        let white = Rgb::rgb(255, 255, 255);
        let dark = darken(white, 0.5);
        assert_eq!(dark.r, 128);
        let mix = blend(Rgb::new(0, 0, 0, 255), white, 0.5);
        assert_eq!(mix.r, 128);
    }

    #[test]
    fn contrast_ignores_premul() {
        let c = contrast(Rgb::new(255, 0, 0, 128));
        assert_eq!(c, Rgb::rgb(0, 0, 0));
    }

    #[test]
    fn palette_preserves_alpha() {
        let p = palette(Rgb::new(255, 0, 0, 128), 3);
        for c in &p {
            assert_eq!(c.a, 128);
        }
    }

    #[test]
    fn to_hex_ignores_premul() {
        assert_eq!(to_hex(Rgb::new(255, 0, 0, 128)), "#ff0000");
    }

    #[test]
    fn complementary_real() {
        let cases = [("402080", "608020"), ("00a0b0", "b01000"), ("f0e68c", "8c96f0")];
        for (input, expected) in cases {
            let base = from_hex(input);
            assert_eq!(to_hex(complementary(base)), format!("#{expected}"));
        }
    }

    #[test]
    fn complementary_preserves_alpha() {
        let comp = complementary(Rgb::new(255, 0, 0, 128));
        assert_eq!(comp.a, 128);
    }

    #[test]
    fn nil_color() {
        assert_eq!(to_hex(Rgb::rgb(0, 0, 0)), "#000000");
    }

    #[test]
    fn composable() {
        // sanity: operations compose without panic
        let c = Rgb::rgb(100, 50, 25);
        let _ = darken(lighten(blend(c, Rgb::rgb(255, 255, 255), 0.5), 0.2), 0.1);
    }
}
