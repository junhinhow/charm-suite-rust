//! Rendering of the wizard progress indicator.
//!
//! Colors and layout match the Go implementation: completed steps are green
//! `[*]`, the current step is yellow `[>]`, pending steps are gray `[ ]`, and a
//! 30-cell bar shows the percentage of steps reached.

use lipgloss::{rounded_border, Color, Style};

use crate::Step;

const COMPLETED: &str = "#04B575";
const CURRENT: &str = "#FBBF24";
const REMAINING: &str = "#6B7280";
const TITLE: &str = "#7C3AED";
const BAR_EMPTY: &str = "#374151";
const STEP_INFO: &str = "#E5E7EB";
const BAR_WIDTH: usize = 30;

fn color(hex: &str) -> Color {
    Color(hex.to_owned())
}

fn completed_style() -> Style {
    Style::new().foreground(color(COMPLETED)).bold(true)
}

fn current_style() -> Style {
    Style::new().foreground(color(CURRENT)).bold(true)
}

fn remaining_style() -> Style {
    Style::new().foreground(color(REMAINING))
}

fn title_style() -> Style {
    Style::new()
        .foreground(color(TITLE))
        .bold(true)
        .padding_left(1)
        .padding_right(1)
        .border(rounded_border())
        .border_foreground(color(TITLE))
}

fn complete_style() -> Style {
    Style::new().foreground(color(COMPLETED)).bold(true)
}

fn bar_filled_style() -> Style {
    Style::new().foreground(color(COMPLETED))
}

fn bar_empty_style() -> Style {
    Style::new().foreground(color(BAR_EMPTY))
}

fn step_info_style() -> Style {
    Style::new().foreground(color(STEP_INFO))
}

/// Renders the progress indicator for `steps`, highlighting the step at `current`.
///
/// The output has three lines: the description of the current step, one dot
/// per step, and the progress bar with its percentage. Returns an empty string
/// when `steps` is empty or `current` is out of range, as the Go
/// implementation does.
///
/// # Examples
///
/// ```
/// use charm_wizard::{render_progress, Step};
///
/// let steps = vec![
///     Step {
///         name: "config".into(),
///         description: "Configuration".into(),
///         run_func: Box::new(|_| Ok(())),
///     },
///     Step {
///         name: "deploy".into(),
///         description: "Deploy".into(),
///         run_func: Box::new(|_| Ok(())),
///     },
/// ];
///
/// let text = render_progress(&steps, 0);
/// assert!(text.contains("Etapa 1 de 2: Configuration"));
/// assert!(text.contains("50%"));
/// ```
pub fn render_progress(steps: &[Step], current: usize) -> String {
    let Some(step) = steps.get(current) else {
        return String::new();
    };
    let info = step_info_style().render(&format!(
        "Etapa {} de {}: {}",
        current + 1,
        steps.len(),
        step.description
    ));
    format!(
        "{info}\n{}\n{}",
        render_dots(steps, current),
        render_bar(steps.len(), current)
    )
}

/// Renders one `[*]`, `[>]` or `[ ]` marker per step, joined by separators.
fn render_dots(steps: &[Step], current: usize) -> String {
    let parts: Vec<String> = steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let (style, marker) = match index.cmp(&current) {
                std::cmp::Ordering::Less => (completed_style(), "[*]"),
                std::cmp::Ordering::Equal => (current_style(), "[>]"),
                std::cmp::Ordering::Greater => (remaining_style(), "[ ]"),
            };
            style.render(&format!("{marker} {}", step.name))
        })
        .collect();
    let separator = remaining_style().render(" --- ");
    parts.join(&separator)
}

/// Renders the progress bar followed by the completed percentage.
///
/// The index is clamped to the last step, so the bar never reports more than
/// 100%. A zero `total` renders an empty bar without a percentage.
fn render_bar(total: usize, current: usize) -> String {
    if total == 0 {
        return bar_empty_style().render(&"░".repeat(BAR_WIDTH));
    }
    let index = current.min(total - 1);
    let filled = (((index + 1) * BAR_WIDTH) / total).min(BAR_WIDTH);
    let percent = (((index + 1) * 100) / total).min(100);
    let full = bar_filled_style().render(&"█".repeat(filled));
    let empty = bar_empty_style().render(&"░".repeat(BAR_WIDTH - filled));
    format!("{full}{empty} {percent}%")
}

/// Renders the bordered title shown at the start of a run.
pub(crate) fn render_title(title: &str) -> String {
    title_style().render(title)
}

/// Renders the success message shown at the end of a run.
pub(crate) fn render_complete(title: &str) -> String {
    complete_style().render(&format!("✓ {title} concluído com sucesso!"))
}

#[cfg(test)]
mod tests {
    use super::{render_bar, render_complete, render_dots, render_progress};
    use crate::Step;

    fn steps(names: &[(&str, &str)]) -> Vec<Step> {
        names
            .iter()
            .map(|(name, description)| Step {
                name: (*name).to_owned(),
                description: (*description).to_owned(),
                run_func: Box::new(|_| Ok(())),
            })
            .collect()
    }

    fn plain(s: &str) -> String {
        charm_core::strip_ansi(s)
    }

    #[test]
    fn dots_mark_completed_current_and_pending() {
        let s = steps(&[("a", "A"), ("b", "B"), ("c", "C")]);
        assert_eq!(plain(&render_dots(&s, 1)), "[*] a --- [>] b --- [ ] c");
    }

    #[test]
    fn bar_reports_percentage_of_steps_reached() {
        assert!(plain(&render_bar(4, 0)).ends_with(" 25%"));
        assert!(plain(&render_bar(4, 3)).ends_with(" 100%"));
    }

    #[test]
    fn bar_is_thirty_cells_wide() {
        let bar = plain(&render_bar(3, 0));
        let cells = bar.chars().filter(|c| *c == '█' || *c == '░').count();
        assert_eq!(cells, 30);
    }

    #[test]
    fn bar_percentage_never_exceeds_one_hundred() {
        for current in 0..=20 {
            let text = plain(&render_bar(4, current));
            let percent: usize = text
                .rsplit(' ')
                .next()
                .and_then(|p| p.trim_end_matches('%').parse().ok())
                .expect("percentage at the end of the bar");
            assert!(percent <= 100, "current={current}: {text:?}");
        }
        assert!(plain(&render_bar(2, 5)).ends_with("100%"));
    }

    #[test]
    fn bar_has_thirty_cells_plus_label_width() {
        assert_eq!(
            lipgloss::utils::width_visible(&render_bar(4, 1)),
            30 + 1 + 3
        );
    }

    #[test]
    fn bar_for_zero_total_is_empty_without_percentage() {
        let text = plain(&render_bar(0, 0));
        assert_eq!(text, "░".repeat(30));
    }

    #[test]
    fn progress_has_info_dots_and_bar_lines() {
        let s = steps(&[("config", "Configuration"), ("deploy", "Deploy")]);
        let lines: Vec<String> = render_progress(&s, 1).lines().map(plain).collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Etapa 2 de 2: Deploy");
        assert_eq!(lines[1], "[*] config --- [>] deploy");
        assert!(lines[2].ends_with(" 100%"));
    }

    #[test]
    fn progress_reports_the_current_step_and_completion_message() {
        let s = steps(&[("a", "Primeira"), ("b", "Segunda"), ("c", "Terceira")]);
        assert!(plain(&render_progress(&s, 1)).contains("Etapa 2 de 3: Segunda"));
        assert_eq!(
            plain(&render_complete("Setup")),
            "✓ Setup concluído com sucesso!"
        );
    }

    #[test]
    fn progress_is_empty_for_no_steps() {
        assert_eq!(render_progress(&[], 0), "");
    }

    #[test]
    fn progress_is_empty_for_out_of_range_index() {
        let s = steps(&[("a", "A")]);
        assert_eq!(render_progress(&s, 1), "");
    }
}
