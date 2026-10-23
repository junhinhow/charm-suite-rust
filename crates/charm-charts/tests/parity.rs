//! Parity tests against the Go original (`reference/go/charm-charts`).
//!
//! The golden file `tests/golden/charts.txt` is produced by the Go program
//! `reference/go/charm-charts/cmd/parity`, which exercises the whole public
//! API and prints a transcript of named blocks. The blocks below mirror that
//! program one to one.

use charm_charts::{BarChart, ProgressBar, Sparkline};
use lipgloss::Style;

/// Formats one named block of the transcript.
fn block(name: &str, content: &str) -> String {
    format!("=== {name} ===\n{content}\n")
}

/// Builds the full transcript, matching the Go program byte for byte.
#[allow(clippy::too_many_lines)]
fn transcript() -> String {
    let d16 = [
        1.0, 2.0, 3.0, 4.0, 5.0, 4.0, 3.0, 2.0, 1.0, 5.0, 7.0, 3.0, 8.0, 2.0, 6.0, 4.0,
    ];
    let d8 = [1.0, 5.0, 3.0, 8.0, 2.0, 9.0, 4.0, 7.0];
    let d10 = [4.0, 2.0, 7.0, 3.0, 8.0, 5.0, 9.0, 1.0, 6.0, 4.0];

    let fruits = ["api", "web", "db"];
    let fruit_values = [12.0, 27.0, 5.0];
    let langs = ["Go", "Rust", "Python", "JS"];
    let lang_values = [85.0, 72.0, 95.0, 68.0];

    let blocks = vec![
        // Sparkline
        block("sparkline_basic", &Sparkline::new(&d16).render()),
        block(
            "sparkline_label",
            &Sparkline::new(&[10.0, 20.0, 15.0, 30.0, 25.0])
                .label("CPU")
                .render(),
        ),
        block(
            "sparkline_width_10",
            &Sparkline::new(&d16).width(10).render(),
        ),
        block("sparkline_width_5", &Sparkline::new(&d16).width(5).render()),
        block("sparkline_width_0", &Sparkline::new(&d16).width(0).render()),
        block(
            "sparkline_width_600",
            &Sparkline::new(&[1.0, 2.0, 3.0]).width(600).render(),
        ),
        block(
            "sparkline_example",
            &Sparkline::new(&d10).width(12).style(Style::new()).render(),
        ),
        block("sparkline_single", &Sparkline::new(&[7.0]).render()),
        block(
            "sparkline_height_3",
            &Sparkline::new(&d8).height(3).render(),
        ),
        block(
            "sparkline_height_3_label",
            &Sparkline::new(&d8).height(3).label("mem").render(),
        ),
        block(
            "sparkline_height_8",
            &Sparkline::new(&d8).height(8).render(),
        ),
        block(
            "sparkline_height_0",
            &Sparkline::new(&d8).height(0).render(),
        ),
        block(
            "sparkline_height_100",
            &Sparkline::new(&d8).height(100).render(),
        ),
        block(
            "sparkline_label_multirow_width_10",
            &Sparkline::new(&d8)
                .width(10)
                .height(2)
                .label("CPU")
                .render(),
        ),
        block("sparkline_empty", &Sparkline::new(&[]).render()),
        block(
            "sparkline_flat",
            &Sparkline::new(&[4.0, 4.0, 4.0, 4.0]).render(),
        ),
        block(
            "sparkline_negative",
            &Sparkline::new(&[-5.0, 0.0, 5.0, -2.0, 3.0]).render(),
        ),
        block(
            "sparkline_fractional",
            &Sparkline::new(&[0.1, 0.25, 0.5, 0.75, 1.0, 0.33]).render(),
        ),
        // BarChart
        block(
            "barchart_horizontal",
            &BarChart::new(&fruits, &fruit_values).render(),
        ),
        block(
            "barchart_horizontal_barwidth_2",
            &BarChart::new(&fruits, &fruit_values).bar_width(2).render(),
        ),
        block(
            "barchart_horizontal_width_30",
            &BarChart::new(&fruits, &fruit_values).width(30).render(),
        ),
        block(
            "barchart_horizontal_no_values",
            &BarChart::new(&fruits, &fruit_values)
                .show_values(false)
                .render(),
        ),
        block(
            "barchart_horizontal_zero_values",
            &BarChart::new(&fruits, &[0.0, 0.0, 0.0]).render(),
        ),
        block(
            "barchart_horizontal_negative",
            &BarChart::new(&fruits, &[-4.0, 8.0, -2.0]).render(),
        ),
        block(
            "barchart_horizontal_fewer_labels",
            &BarChart::new(&["api"], &fruit_values).render(),
        ),
        block(
            "barchart_horizontal_more_labels",
            &BarChart::new(&["api", "web", "db", "cache"], &[12.0, 27.0]).render(),
        ),
        block(
            "barchart_horizontal_width_10",
            &BarChart::new(&fruits, &fruit_values).width(10).render(),
        ),
        block(
            "barchart_horizontal_width_20_long_label",
            &BarChart::new(&["Programação", "Go"], &[85.0, 72.0])
                .width(20)
                .render(),
        ),
        block(
            "barchart_horizontal_unicode",
            &BarChart::new(
                &["Ação", "日本語", "Go", "Programação"],
                &[10.0, 20.0, 30.0, 40.0],
            )
            .width(30)
            .render(),
        ),
        block(
            "barchart_horizontal_barwidth_0",
            &BarChart::new(&["a"], &[1.0]).bar_width(0).render(),
        ),
        block(
            "barchart_example",
            &BarChart::new(&["Go", "Rust"], &[85.0, 72.0])
                .width(30)
                .style(Style::new())
                .bar_style(Style::new())
                .label_style(Style::new())
                .render(),
        ),
        block(
            "barchart_readme",
            &BarChart::new(&langs, &lang_values).width(50).render(),
        ),
        block(
            "barchart_vertical",
            &BarChart::new(&["a", "b", "c"], &[3.0, 7.0, 5.0])
                .vertical()
                .render(),
        ),
        block(
            "barchart_vertical_barwidth_3",
            &BarChart::new(&["a", "b", "c"], &[3.0, 7.0, 5.0])
                .vertical()
                .bar_width(3)
                .render(),
        ),
        block(
            "barchart_vertical_long_labels",
            &BarChart::new(&["alpha", "beta", "gamma"], &[3.0, 7.0, 5.0])
                .vertical()
                .render(),
        ),
        block(
            "barchart_vertical_negative",
            &BarChart::new(&["a", "b", "c"], &[-3.0, 7.0, 5.0])
                .vertical()
                .render(),
        ),
        block("barchart_empty", &BarChart::new(&["a"], &[]).render()),
        block(
            "barchart_toggle_vertical_last",
            &BarChart::new(&["a", "b"], &[3.0, 7.0])
                .vertical()
                .horizontal()
                .render(),
        ),
        block(
            "barchart_vertical_width_20",
            &BarChart::new(&langs, &lang_values)
                .width(20)
                .vertical()
                .bar_width(3)
                .render(),
        ),
        block(
            "barchart_vertical_width_60",
            &BarChart::new(&langs, &lang_values)
                .width(60)
                .vertical()
                .bar_width(3)
                .render(),
        ),
        block(
            "barchart_vertical_width_50",
            &BarChart::new(&langs, &lang_values)
                .width(50)
                .vertical()
                .bar_width(3)
                .render(),
        ),
        block(
            "barchart_vertical_hidden_values",
            &BarChart::new(&["Go", "Rust"], &[85.0, 72.0])
                .vertical()
                .show_values(false)
                .render(),
        ),
        block(
            "barchart_vertical_unicode",
            &BarChart::new(
                &["Ação", "日本語", "Go", "Programação"],
                &[10.0, 20.0, 30.0, 40.0],
            )
            .width(20)
            .vertical()
            .render(),
        ),
        block(
            "barchart_vertical_long_label_width_20",
            &BarChart::new(&["Programação"], &[10.0])
                .width(20)
                .vertical()
                .render(),
        ),
        // ProgressBar
        block("progress_default", &ProgressBar::new(0.75).render()),
        block(
            "progress_label",
            &ProgressBar::new(0.5).label("download").render(),
        ),
        block(
            "progress_no_percentage",
            &ProgressBar::new(0.5).show_percentage(false).render(),
        ),
        block("progress_zero", &ProgressBar::new(0.0).render()),
        block("progress_one", &ProgressBar::new(1.0).render()),
        block("progress_above_one", &ProgressBar::new(2.5).render()),
        block("progress_below_zero", &ProgressBar::new(-0.5).render()),
        block("progress_width_0", &ProgressBar::new(0.5).width(0).render()),
        block(
            "progress_width_10",
            &ProgressBar::new(0.42).width(10).render(),
        ),
        block(
            "progress_width_clamped",
            &ProgressBar::new(0.5).width(600).render(),
        ),
        block(
            "progress_custom_chars",
            &ProgressBar::new(0.42)
                .fill_char('#')
                .empty_char('.')
                .render(),
        ),
        block("progress_fractional", &ProgressBar::new(0.333).render()),
        block(
            "progress_example",
            &ProgressBar::new(0.73).width(30).label("Upload").render(),
        ),
    ];

    blocks.join("\n")
}

#[test]
fn charts_match_the_go_implementation() {
    charm_parity::assert_golden("charts", &transcript());
}
