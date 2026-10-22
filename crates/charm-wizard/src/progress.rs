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
/// when `steps` is empty or `current` is out of range; the Go implementation
/// panics in the latter case.
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
/// assert!(text.contains("Step 1 of 2: Configuration"));
/// assert!(text.contains("50%"));
/// ```
pub fn render_progress(steps: &[Step], current: usize) -> String {
    let Some(step) = steps.get(current) else {
        return String::new();
    };
    let info = step_info_style().render(&format!(
        "Step {} of {}: {}",
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
fn render_bar(total: usize, current: usize) -> String {
    let (filled, percent) = match total {
        0 => (0, 0),
        _ => (
            (((current + 1) * BAR_WIDTH) / total).min(BAR_WIDTH),
            ((current + 1) * 100) / total,
        ),
    };
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
    complete_style().render(&format!("✓ {title} completed successfully!"))
}

#[cfg(test)]
mod tests {
    use super::{render_bar, render_dots, render_progress};
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
    fn progress_has_info_dots_and_bar_lines() {
        let s = steps(&[("config", "Configuration"), ("deploy", "Deploy")]);
        let lines: Vec<String> = render_progress(&s, 1).lines().map(plain).collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Step 2 of 2: Deploy");
        assert_eq!(lines[1], "[*] config --- [>] deploy");
        assert!(lines[2].ends_with(" 100%"));
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
