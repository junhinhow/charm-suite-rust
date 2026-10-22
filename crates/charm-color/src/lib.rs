//! Color manipulation utilities for terminal user interfaces.
//!
//! `charm-color` is a Rust port of the Go library
//! [`charm-color`](https://github.com/junhinhow/charm-color). It provides the
//! color operations that terminal UIs need most: darkening, lightening,
//! blending, picking a readable contrast color, deriving the complementary
//! color and building harmonious palettes.
//!
//! Every function reproduces the original Go implementation bit-for-bit,
//! including its truncation and rounding rules, so the same inputs always
//! produce the same bytes.
//!
//! # Example
//!
//! ```
//! use charm_color::{blend, complementary, contrast, darken, lighten, palette};
//! use charm_core::Rgb;
//!
//! let red = Rgb::rgb(255, 0, 0);
//!
//! assert_eq!(darken(red, 0.5).to_hex(), "#7f0000");
//! assert_eq!(lighten(red, 0.5).to_hex(), "#ff7f7f");
//! assert_eq!(blend(Rgb::black(), Rgb::white(), 0.5).to_hex(), "#7f7f7f");
//! assert_eq!(contrast(red).to_hex(), "#000000");
//! assert_eq!(complementary(red).to_hex(), "#00ffff");
//! assert_eq!(palette(red, 2).len(), 2);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use charm_core::Rgb;

/// Darkens a color by the given amount.
///
/// The `amount` is clamped to `0.0..=1.0`: `0.0` keeps the color unchanged
/// and `1.0` turns it black. The alpha channel is preserved.
///
/// The Go original truncates the resulting channels instead of rounding them,
/// which this port reproduces exactly.
///
/// # Examples
///
/// ```
/// use charm_color::darken;
/// use charm_core::Rgb;
///
/// let base = Rgb::rgb(200, 100, 50);
/// assert_eq!(darken(base, 0.0).to_hex(), "#c86432");
/// assert_eq!(darken(base, 0.5).to_hex(), "#643219");
/// assert_eq!(darken(base, 1.0).to_hex(), "#000000");
/// ```
///
/// Amounts outside `0.0..=1.0` are clamped:
///
/// ```
/// use charm_color::darken;
/// use charm_core::Rgb;
///
/// let base = Rgb::rgb(200, 100, 50);
/// assert_eq!(darken(base, 2.0), darken(base, 1.0));
/// assert_eq!(darken(base, -1.0), darken(base, 0.0));
/// ```
pub fn darken(color: Rgb, amount: f64) -> Rgb {
    let (red, green, blue, alpha) = rgba16(color);
    let factor = clamp(1.0 - amount);
    Rgb::new(
        (f64::from(red >> 8) * factor) as u8,
        (f64::from(green >> 8) * factor) as u8,
        (f64::from(blue >> 8) * factor) as u8,
        (alpha >> 8) as u8,
    )
}

/// Lightens a color by the given amount.
///
/// The `amount` is clamped to `0.0..=1.0`: `0.0` keeps the color unchanged
/// and `1.0` turns it white. The alpha channel is preserved.
///
/// Like the Go original, the resulting channels are truncated (not rounded).
///
/// # Examples
///
/// ```
/// use charm_color::lighten;
/// use charm_core::Rgb;
///
/// let base = Rgb::rgb(100, 50, 25);
/// assert_eq!(lighten(base, 0.0).to_hex(), "#643219");
/// assert_eq!(lighten(base, 0.5).to_hex(), "#b1988c");
/// assert_eq!(lighten(base, 1.0).to_hex(), "#ffffff");
/// ```
pub fn lighten(color: Rgb, amount: f64) -> Rgb {
    let (red, green, blue, alpha) = rgba16(color);
    let red = f64::from(red >> 8);
    let green = f64::from(green >> 8);
    let blue = f64::from(blue >> 8);
    let amount = clamp(amount);
    Rgb::new(
        (red + (255.0 - red) * amount) as u8,
        (green + (255.0 - green) * amount) as u8,
        (blue + (255.0 - blue) * amount) as u8,
        (alpha >> 8) as u8,
    )
}

/// Interpolates between two colors.
///
/// `t` is clamped to `0.0..=1.0`: `0.0` returns `c1` and `1.0` returns `c2`.
/// The alpha channel is interpolated as well.
///
/// # Examples
///
/// ```
/// use charm_color::blend;
/// use charm_core::Rgb;
///
/// let black = Rgb::black();
/// let white = Rgb::white();
/// assert_eq!(blend(black, white, 0.0).to_hex(), "#000000");
/// assert_eq!(blend(black, white, 0.5).to_hex(), "#7f7f7f");
/// assert_eq!(blend(black, white, 1.0).to_hex(), "#ffffff");
/// ```
pub fn blend(c1: Rgb, c2: Rgb, t: f64) -> Rgb {
    let t = clamp(t);
    let (r1, g1, b1, a1) = rgba16(c1);
    let (r2, g2, b2, a2) = rgba16(c2);
    Rgb::new(
        lerp(r1 >> 8, r2 >> 8, t) as u8,
        lerp(g1 >> 8, g2 >> 8, t) as u8,
        lerp(b1 >> 8, b2 >> 8, t) as u8,
        lerp(a1 >> 8, a2 >> 8, t) as u8,
    )
}

/// Returns black or white, whichever is more readable on top of `color`.
///
/// The decision is made with the WCAG relative luminance formula: colors
/// brighter than `0.179` get black text, the others get white text. The
/// returned color is always fully opaque.
///
/// # Examples
///
/// ```
/// use charm_color::contrast;
/// use charm_core::Rgb;
///
/// assert_eq!(contrast(Rgb::white()).to_hex(), "#000000");
/// assert_eq!(contrast(Rgb::black()).to_hex(), "#ffffff");
/// assert_eq!(contrast(Rgb::rgb(255, 0, 0)).to_hex(), "#000000");
/// ```
pub fn contrast(color: Rgb) -> Rgb {
    let (red, green, blue, _) = rgba16(color);
    let luminance = 0.2126 * srgb_to_linear(f64::from(red >> 8) / 255.0)
        + 0.7152 * srgb_to_linear(f64::from(green >> 8) / 255.0)
        + 0.0722 * srgb_to_linear(f64::from(blue >> 8) / 255.0);
    if luminance > 0.179 {
        Rgb::black()
    } else {
        Rgb::white()
    }
}

/// Returns the complementary (opposite) color.
///
/// The alpha channel is preserved.
///
/// # Examples
///
/// ```
/// use charm_color::complementary;
/// use charm_core::Rgb;
///
/// assert_eq!(complementary(Rgb::rgb(255, 0, 0)).to_hex(), "#00ffff");
/// assert_eq!(complementary(Rgb::black()).to_hex(), "#ffffff");
/// ```
pub fn complementary(color: Rgb) -> Rgb {
    let (red, green, blue, alpha) = rgba16(color);
    Rgb::new(
        (255 - (red >> 8)) as u8,
        (255 - (green >> 8)) as u8,
        (255 - (blue >> 8)) as u8,
        (alpha >> 8) as u8,
    )
}

/// Builds a harmonious palette of `n` colors from a base color.
///
/// The base color is converted to HSL and the hue is rotated uniformly, so
/// every color of the palette keeps the saturation and lightness of the base.
/// A count of zero returns an empty palette.
///
/// # Examples
///
/// ```
/// use charm_color::palette;
/// use charm_core::Rgb;
///
/// let colors = palette(Rgb::rgb(255, 0, 0), 4);
/// assert_eq!(colors.len(), 4);
/// assert_eq!(colors[0].to_hex(), "#ff0000");
/// assert_eq!(colors[1].to_hex(), "#80ff00");
/// assert_eq!(colors[2].to_hex(), "#00ffff");
/// assert_eq!(colors[3].to_hex(), "#7f00ff");
///
/// assert!(palette(Rgb::rgb(255, 0, 0), 0).is_empty());
/// ```
pub fn palette(base: Rgb, n: usize) -> Vec<Rgb> {
    if n == 0 {
        return Vec::new();
    }
    let (hue, saturation, lightness) = rgb_to_hsl(base);
    (0..n)
        .map(|index| {
            let rotated = (hue + (index as f64) * (360.0 / n as f64)) % 360.0;
            hsl_to_rgb(rotated, saturation, lightness)
        })
        .collect()
}

/// Converts a lipgloss color string into an [`Rgb`].
///
/// Accepted formats are `"#rrggbb"`, `"#rgb"` and the ANSI-256 palette
/// indexes (`"0"`..=`"255"`). Invalid strings fall back to opaque black,
/// exactly like the Go original.
///
/// # Examples
///
/// ```
/// use charm_color::from_lipgloss;
///
/// assert_eq!(from_lipgloss("#ff8000").to_hex(), "#ff8000");
/// assert_eq!(from_lipgloss("#fff").to_hex(), "#ffffff");
/// assert_eq!(from_lipgloss("63").to_hex(), "#5f5fff");
/// assert_eq!(from_lipgloss("nope").to_hex(), "#000000");
/// ```
pub fn from_lipgloss(s: &str) -> Rgb {
    Rgb::from_lipgloss(&lipgloss::Color(s.to_owned()))
}

/// Converts a color into the 16-bit premultiplied channels used by Go's
/// `color.NRGBA.RGBA()`.
///
/// The Go implementation always works on premultiplied values, so colors
/// with a translucent alpha channel behave slightly differently from their
/// opaque counterparts. Reproducing that conversion here keeps the Rust port
/// byte-compatible with the original.
fn rgba16(color: Rgb) -> (u32, u32, u32, u32) {
    let alpha = u32::from(color.a);
    (
        premultiply(color.r, color.a),
        premultiply(color.g, color.a),
        premultiply(color.b, color.a),
        alpha | (alpha << 8),
    )
}

/// Premultiplies a single 8-bit channel by the alpha channel.
fn premultiply(value: u8, alpha: u8) -> u32 {
    let mut channel = u32::from(value);
    channel |= channel << 8;
    channel *= u32::from(alpha);
    channel / 0xff
}

/// Clamps a value to the `0.0..=1.0` range.
///
/// `NaN` is returned unchanged, exactly like the Go implementation.
fn clamp(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

/// Interpolates between two 8-bit channels.
fn lerp(a: u32, b: u32, t: f64) -> f64 {
    let a = f64::from(a);
    let b = f64::from(b);
    a + (b - a) * t
}

/// Converts an sRGB channel (0.0..=1.0) to the linear space.
fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts a color to the HSL representation.
///
/// The comparisons are exact on purpose: they mirror the Go implementation,
/// which relies on the bit-exact equality of the normalized channels.
#[allow(clippy::float_cmp)]
fn rgb_to_hsl(color: Rgb) -> (f64, f64, f64) {
    let (red, green, blue, _) = rgba16(color);
    let red = f64::from(red >> 8) / 255.0;
    let green = f64::from(green >> 8) / 255.0;
    let blue = f64::from(blue >> 8) / 255.0;

    let max = red.max(green.max(blue));
    let min = red.min(green.min(blue));
    let lightness = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, lightness);
    }

    let delta = max - min;
    let saturation = if lightness > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    let hue = if max == red {
        let hue = (green - blue) / delta;
        if green < blue {
            hue + 6.0
        } else {
            hue
        }
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };

    (hue * 60.0, saturation, lightness)
}

/// Converts an HSL color to RGB.
///
/// The gray case truncates the channel while the colored case rounds it,
/// exactly like the Go implementation.
#[allow(clippy::float_cmp)]
fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> Rgb {
    if saturation == 0.0 {
        let value = (lightness * 255.0) as u8;
        return Rgb::new(value, value, value, 255);
    }

    let brightness = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let base = 2.0 * lightness - brightness;

    let normalized = hue / 360.0;
    let red = hue_to_rgb(base, brightness, normalized + 1.0 / 3.0);
    let green = hue_to_rgb(base, brightness, normalized);
    let blue = hue_to_rgb(base, brightness, normalized - 1.0 / 3.0);

    Rgb::new(
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
        255,
    )
}

/// Computes a single channel of the HSL to RGB conversion.
fn hue_to_rgb(base: f64, brightness: f64, position: f64) -> f64 {
    let mut position = position;
    if position < 0.0 {
        position += 1.0;
    }
    if position > 1.0 {
        position -= 1.0;
    }
    if position < 1.0 / 6.0 {
        return base + (brightness - base) * 6.0 * position;
    }
    if position < 1.0 / 2.0 {
        return brightness;
    }
    if position < 2.0 / 3.0 {
        return base + (brightness - base) * (2.0 / 3.0 - position) * 6.0;
    }
    base
}

#[cfg(test)]
mod tests {
    //! Unit tests ported one-to-one from the Go `color_test.go`.
    //!
    //! Every test below keeps the name, the inputs and the expectations of
    //! the original test, so the two suites can be compared line by line.

    use super::*;

    /// Go: `TestDarken`.
    #[test]
    fn test_darken() {
        let color = Rgb::new(200, 100, 50, 255);
        let darkened = darken(color, 0.5);
        assert!(darkened.r <= 200, "Darken deveria reduzir os valores RGB");
        assert!(darkened.g <= 100, "Darken deveria reduzir os valores RGB");
        assert!(darkened.b <= 50, "Darken deveria reduzir os valores RGB");
    }

    /// Go: `TestLighten`.
    #[test]
    fn test_lighten() {
        let color = Rgb::new(100, 50, 25, 255);
        let lightened = lighten(color, 0.5);
        assert!(
            lightened.r >= 100,
            "Lighten deveria aumentar os valores RGB"
        );
        assert!(lightened.g >= 50, "Lighten deveria aumentar os valores RGB");
        assert!(lightened.b >= 25, "Lighten deveria aumentar os valores RGB");
    }

    /// Go: `TestBlend`.
    #[test]
    fn test_blend() {
        let black = Rgb::new(0, 0, 0, 255);
        let white = Rgb::new(255, 255, 255, 255);
        let mid = blend(black, white, 0.5);
        assert!(
            (120..=135).contains(&mid.r),
            "Blend(black, white, 0.5) esperado ~127, obtido {}",
            mid.r
        );
    }

    /// Go: `TestContrastDark`.
    #[test]
    fn contrast_dark() {
        let white = Rgb::new(255, 255, 255, 255);
        let contrasted = contrast(white);
        assert!(
            contrasted.r == 0 && contrasted.g == 0 && contrasted.b == 0,
            "Contrast de branco deveria retornar preto"
        );
    }

    /// Go: `TestContrastLight`.
    #[test]
    fn contrast_light() {
        let black = Rgb::new(0, 0, 0, 255);
        let contrasted = contrast(black);
        assert!(
            contrasted.r == 255 && contrasted.g == 255 && contrasted.b == 255,
            "Contrast de preto deveria retornar branco"
        );
    }

    /// Go: `TestComplementary`.
    #[test]
    fn test_complementary() {
        let color = Rgb::new(255, 0, 0, 255);
        let complementary = complementary(color);
        assert!(
            complementary.r == 0 && complementary.g == 255 && complementary.b == 255,
            "Complementar de vermelho deveria ser ciano, obtido ({},{},{})",
            complementary.r,
            complementary.g,
            complementary.b
        );
    }

    /// Go: `TestPalette`.
    #[test]
    fn test_palette() {
        let base = Rgb::new(255, 0, 0, 255);
        let palette = palette(base, 4);
        assert_eq!(palette.len(), 4, "Palette deveria ter 4 cores");
    }

    /// Go: `TestPaletteZero`.
    #[test]
    fn palette_zero() {
        let base = Rgb::new(255, 0, 0, 255);
        let palette = palette(base, 0);
        assert!(palette.is_empty(), "Palette(0) deveria retornar nil");
    }

    /// Go: `TestToHex`.
    #[test]
    fn to_hex() {
        let color = Rgb::new(255, 128, 0, 255);
        assert_eq!(color.to_hex(), "#ff8000");
    }

    /// Go: `TestFromHex`.
    #[test]
    fn from_hex() {
        let color = Rgb::from_hex_or_black("#ff8000");
        assert!(
            color.r == 255 && color.g == 128 && color.b == 0,
            "FromHex #ff8000 esperado (255,128,0), obtido ({},{},{})",
            color.r,
            color.g,
            color.b
        );
    }

    /// Go: `TestFromHexWithoutHash`.
    #[test]
    fn from_hex_without_hash() {
        let color = Rgb::from_hex_or_black("00ff00");
        assert!(
            color.r == 0 && color.g == 255 && color.b == 0,
            "FromHex 00ff00 esperado (0,255,0), obtido ({},{},{})",
            color.r,
            color.g,
            color.b
        );
    }

    /// Go: `TestFromHexInvalid`.
    #[test]
    fn from_hex_invalid() {
        let color = Rgb::from_hex_or_black("invalid");
        assert!(
            color.r == 0 && color.g == 0 && color.b == 0,
            "FromHex invalido deveria retornar preto"
        );
    }

    // --- Extra coverage for the edges of the Go implementation. ---

    #[test]
    fn darken_truncates_instead_of_rounding() {
        // 255 * 0.5 = 127.5 -> 127 (truncation), not 128 (rounding).
        assert_eq!(darken(Rgb::rgb(255, 255, 255), 0.5).to_hex(), "#7f7f7f");
    }

    #[test]
    fn lighten_truncates_instead_of_rounding() {
        // 0 + 255 * 0.5 = 127.5 -> 127.
        assert_eq!(lighten(Rgb::black(), 0.5).to_hex(), "#7f7f7f");
    }

    #[test]
    fn amounts_are_clamped() {
        let base = Rgb::rgb(200, 100, 50);
        assert_eq!(darken(base, 5.0), darken(base, 1.0));
        assert_eq!(darken(base, -5.0), darken(base, 0.0));
        assert_eq!(lighten(base, 5.0), lighten(base, 1.0));
        assert_eq!(lighten(base, -5.0), lighten(base, 0.0));
        assert_eq!(
            blend(base, Rgb::white(), 5.0),
            blend(base, Rgb::white(), 1.0)
        );
        assert_eq!(
            blend(base, Rgb::white(), -5.0),
            blend(base, Rgb::white(), 0.0)
        );
    }

    #[test]
    fn alpha_is_preserved() {
        let translucent = Rgb::new(200, 100, 50, 128);
        assert_eq!(darken(translucent, 0.5).a, 128);
        assert_eq!(lighten(translucent, 0.5).a, 128);
        assert_eq!(complementary(translucent).a, 128);
    }

    #[test]
    fn translucent_colors_use_premultiplied_channels() {
        // Go's color.NRGBA.RGBA() premultiplies the channels by the alpha.
        assert_eq!(darken(Rgb::new(12, 34, 56, 128), 0.4).to_hex(), "#030a10");
        assert_eq!(lighten(Rgb::new(12, 34, 56, 128), 0.4).to_hex(), "#697076");
    }

    #[test]
    fn blend_interpolates_alpha() {
        let blended = blend(Rgb::new(10, 20, 30, 128), Rgb::new(40, 50, 60, 255), 0.5);
        assert_eq!(blended.to_hex(), "#161e25");
        assert_eq!(blended.a, 191);
    }

    #[test]
    fn palette_is_gray_for_gray_base() {
        let palette = palette(Rgb::rgb(128, 128, 128), 3);
        assert!(palette.iter().all(|c| c.to_hex() == "#808080"));
    }

    #[test]
    fn palette_rotates_the_hue_evenly() {
        let palette = palette(Rgb::rgb(255, 0, 255), 4);
        let hexes: Vec<String> = palette.iter().map(|c| c.to_hex()).collect();
        assert_eq!(hexes, ["#ff00ff", "#ff8000", "#00ff00", "#007fff"]);
    }

    #[test]
    fn palette_keeps_the_saturation_of_light_bases() {
        let palette = palette(Rgb::rgb(250, 189, 47), 5);
        assert_eq!(palette[0].to_hex(), "#fabd2f");
        assert_eq!(palette[4].to_hex(), "#fa2f95");
    }

    #[test]
    fn from_lipgloss_accepts_hex_and_ansi() {
        assert_eq!(from_lipgloss("#ff8000").to_hex(), "#ff8000");
        assert_eq!(from_lipgloss("#fff").to_hex(), "#ffffff");
        assert_eq!(from_lipgloss("63").to_hex(), "#5f5fff");
        assert_eq!(from_lipgloss("5").to_hex(), "#800080");
        assert_eq!(from_lipgloss("255").to_hex(), "#eeeeee");
        assert_eq!(from_lipgloss("invalid").to_hex(), "#000000");
        assert_eq!(from_lipgloss("").to_hex(), "#000000");
        assert_eq!(from_lipgloss("#zzz").to_hex(), "#000000");
    }
}
