use lipgloss::{Border, Style};

use crate::ascii::transliterate_static;
use crate::profile::Profile;

const ZERO_BORDER: Border = Border::new("", "", "", "", "", "", "", "", "", "", "", "", "");

impl Profile {
    /// Degrades `style` to suit the profile.
    ///
    /// Without colors, foreground, background, border and margin colors are
    /// removed. With high contrast, `faint` is removed and `bold` is applied,
    /// without adding any color. Without Unicode, a set border is replaced by
    /// its ASCII version, keeping the enabled sides. The input is consumed
    /// and no other state is changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    /// use lipgloss::{normal_border, Style};
    ///
    /// let profile = Profile::default();
    /// let style = Style::new().border(normal_border());
    /// let out = profile.style(style).render("ok");
    /// assert_eq!(out, "+--+\n|ok|\n+--+");
    /// ```
    pub fn style(&self, style: Style) -> Style {
        let mut s = style;
        if self.no_color {
            s = s
                .unset_foreground()
                .unset_background()
                .unset_border_foreground()
                .unset_border_background()
                .unset_margin_background();
        }
        if self.high_contrast {
            s = s.unset_faint().bold(true);
        }
        if !self.unicode {
            let border = s.get_border_style();
            if border != ZERO_BORDER {
                s = s.border_style(self.border(border));
            }
        }
        s
    }

    /// Returns a border that suits the profile.
    ///
    /// With Unicode, `border` is returned unchanged. Otherwise each part is
    /// transliterated with the same table as [`Profile::sanitize`]. Custom
    /// parts with unknown glyphs are kept.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    /// use lipgloss::rounded_border;
    ///
    /// let border = Profile::default().border(rounded_border());
    /// assert_eq!(border.top_left, "+");
    /// assert_eq!(border.top, "-");
    /// ```
    pub fn border(&self, border: Border) -> Border {
        if self.unicode {
            return border;
        }
        Border::new(
            transliterate_static(border.top),
            transliterate_static(border.bottom),
            transliterate_static(border.left),
            transliterate_static(border.right),
            transliterate_static(border.top_left),
            transliterate_static(border.top_right),
            transliterate_static(border.bottom_left),
            transliterate_static(border.bottom_right),
            transliterate_static(border.middle_left),
            transliterate_static(border.middle_right),
            transliterate_static(border.middle),
            transliterate_static(border.middle_top),
            transliterate_static(border.middle_bottom),
        )
    }
}

#[cfg(test)]
mod tests {
    use lipgloss::renderer::{set_color_profile, ColorProfileKind};
    use lipgloss::{
        ascii_border, block_border, double_border, normal_border, outer_half_block_border,
        rounded_border, thick_border, Border, Style,
    };

    use crate::profile::Profile;

    const ASCII_BORDER: Border = Border::new(
        "-", "-", "|", "|", "+", "+", "+", "+", "+", "+", "+", "+", "+",
    );

    fn truecolor() {
        set_color_profile(ColorProfileKind::TrueColor);
    }

    fn no_color_unicode() -> Profile {
        Profile {
            no_color: true,
            unicode: true,
            ..Profile::default()
        }
    }

    const COLOR_PREFIXES: [&str; 3] = ["\u{1b}[38;", "\u{1b}[48;", "\u{1b}[58;"];

    #[test]
    fn no_color_removes_colors() {
        truecolor();
        let base = Style::new()
            .foreground("#ff0000")
            .background("#00ff00")
            .border(normal_border())
            .border_foreground("#0000ff")
            .margin_background("#111111")
            .margin(0, 1, 0, 1);

        assert!(
            base.render("x").contains("\u{1b}["),
            "control style should contain ANSI"
        );

        let styled = no_color_unicode().style(base.clone());
        assert!(styled.get_foreground().is_none());
        assert!(styled.get_background().is_none());
        assert!(styled.get_margin_background().is_none());

        let rendered = styled.render("x");
        assert!(
            !rendered.contains("\u{1b}["),
            "no-color style still has ANSI: {rendered:?}"
        );
        assert!(
            rendered.contains('x'),
            "no-color style lost the text: {rendered:?}"
        );
        assert!(
            rendered.contains('─'),
            "unicode should keep the border: {rendered:?}"
        );

        assert!(
            base.render("x").contains("\u{1b}["),
            "style modified the input"
        );
    }

    #[test]
    fn no_color_keeps_underline_and_removes_all_color() {
        truecolor();
        let base = Style::new()
            .underline(true)
            .foreground("#ff0000")
            .border(normal_border())
            .border_foreground("#00ff00");

        let styled = no_color_unicode().style(base);
        let rendered = styled.render("x");
        for prefix in COLOR_PREFIXES {
            assert!(
                !rendered.contains(prefix),
                "no-color left color ({prefix:?}): {rendered:?}"
            );
        }
        assert!(styled.get_underline(), "no-color should keep underline");
    }

    #[test]
    fn high_contrast() {
        truecolor();
        let base = Style::new().faint(true);
        let styled = Profile {
            high_contrast: true,
            unicode: true,
            ..Profile::default()
        }
        .style(base);

        assert!(!styled.get_faint(), "high contrast should remove faint");
        assert!(styled.get_bold(), "high contrast should apply bold");
        assert!(styled.get_foreground().is_none());
        assert!(styled.get_background().is_none());

        let rendered = styled.render("x");
        assert!(
            rendered.contains("\u{1b}[1m"),
            "output without bold: {rendered:?}"
        );
        for prefix in COLOR_PREFIXES {
            assert!(
                !rendered.contains(prefix),
                "high contrast added color: {rendered:?}"
            );
        }
    }

    #[test]
    fn no_color_and_high_contrast() {
        truecolor();
        let base = Style::new()
            .foreground("#ff0000")
            .faint(true)
            .border(rounded_border())
            .border_foreground("#00ff00");

        let styled = Profile {
            no_color: true,
            high_contrast: true,
            unicode: true,
            ..Profile::default()
        }
        .style(base);
        assert!(styled.get_bold(), "bold should be kept");
        assert!(!styled.get_faint(), "faint should be removed");

        let rendered = styled.render("x");
        for prefix in COLOR_PREFIXES {
            assert!(
                !rendered.contains(prefix),
                "combination added color: {rendered:?}"
            );
        }
    }

    #[test]
    fn style_degrades_border_to_ascii() {
        let base = Style::new().border(rounded_border());
        let styled = Profile::default().style(base);

        assert_eq!(styled.get_border_style(), ASCII_BORDER);

        let rendered = styled.render("x");
        assert!(
            !rendered.contains(['╭', '╮', '╰', '╯', '─', '│']),
            "unicode border left: {rendered:?}"
        );
        for part in ["+", "-", "|"] {
            assert!(
                rendered.contains(part),
                "ASCII border without {part:?}: {rendered:?}"
            );
        }
    }

    #[test]
    fn preserves_border_sides() {
        let base = Style::new().border_with_sides(normal_border(), &[true, false, false, false]);
        let styled = Profile::default().style(base);

        assert!(styled.get_border_top(), "top border should stay enabled");
        assert!(!styled.get_border_right());
        assert!(!styled.get_border_bottom());
        assert!(!styled.get_border_left());
    }

    #[test]
    fn unicode_border_unchanged() {
        let base = Style::new().border(rounded_border());
        let styled = Profile {
            unicode: true,
            ..Profile::default()
        }
        .style(base);
        assert_eq!(styled.get_border_style(), rounded_border());
    }

    #[test]
    fn border_cases() {
        let unicode = Profile {
            unicode: true,
            ..Profile::default()
        };
        let ascii = Profile::default();
        let double_ascii = Border::new(
            "=", "=", "|", "|", "+", "+", "+", "+", "+", "+", "+", "+", "+",
        );
        let block_ascii = Border::new(
            "#", "#", "#", "#", "#", "#", "#", "#", "#", "#", "#", "#", "#",
        );
        let half_block_ascii =
            Border::new("^", "_", "[", "]", "+", "+", "+", "+", "", "", "", "", "");
        let cases: &[(Profile, Border, Border)] = &[
            (unicode, normal_border(), normal_border()),
            (unicode, rounded_border(), rounded_border()),
            (ascii, normal_border(), ASCII_BORDER),
            (ascii, rounded_border(), ASCII_BORDER),
            (ascii, thick_border(), ASCII_BORDER),
            (ascii, double_border(), double_ascii),
            (ascii, block_border(), block_ascii),
            (ascii, outer_half_block_border(), half_block_ascii),
            (
                ascii,
                Border::new("", "", "", "", "", "", "", "", "", "", "", "", ""),
                Border::new("", "", "", "", "", "", "", "", "", "", "", "", ""),
            ),
        ];
        for (profile, input, want) in cases {
            assert_eq!(profile.border(*input), *want, "input {input:?}");
        }
        assert_eq!(ascii.border(ascii_border()), ASCII_BORDER);
    }

    #[test]
    fn style_zero_value_keeps_color_and_degrades_border() {
        let base = Style::new().foreground("#ff0000").border(rounded_border());
        let got = Profile::default().style(base);
        assert!(
            got.get_foreground().is_some(),
            "zero profile should keep the color"
        );
        assert_ne!(
            got.get_border_style(),
            rounded_border(),
            "zero profile should convert the border"
        );
    }
}
