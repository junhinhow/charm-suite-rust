//! Demo of the `charm-charts` crate.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-charts --example charm_charts_demo
//! ```

use charm_charts::{BarChart, ProgressBar, Sparkline};
use lipgloss::{Color, Style};

fn main() {
    let title = Style::new().bold(true).foreground(Color("212".to_string()));

    println!("{}", title.render("charm-charts demo"));
    println!();

    // ------------------------------------------------------------------
    // Sparklines
    // ------------------------------------------------------------------
    println!("{}", title.render("Sparklines"));

    let cpu = Sparkline::new(&[12.0, 18.0, 9.0, 24.0, 31.0, 27.0, 15.0, 8.0, 19.0, 22.0]);
    println!("cpu {}", cpu.render());

    let memory = Sparkline::new(&[40.0, 44.0, 51.0, 48.0, 62.0, 71.0, 66.0, 59.0, 54.0, 50.0])
        .label("mem")
        .style(Style::new().foreground(Color("42".to_string())));
    println!("{}", memory.render());

    let tall = Sparkline::new(&[3.0, 9.0, 5.0, 12.0, 7.0, 15.0, 11.0, 4.0]).height(4);
    println!("{}", tall.render());

    println!();

    // ------------------------------------------------------------------
    // Bar charts
    // ------------------------------------------------------------------
    println!("{}", title.render("Bar chart (horizontal)"));
    let horizontal = BarChart::new(
        &["api", "web", "worker", "cache"],
        &[128.0, 342.0, 76.0, 251.0],
    );
    println!("{}", horizontal.render());
    println!();

    println!("{}", title.render("Bar chart (vertical)"));
    let vertical = BarChart::new(&["a", "b", "c", "d"], &[12.0, 30.0, 21.0, 8.0]).vertical();
    println!("{}", vertical.render());
    println!();

    // ------------------------------------------------------------------
    // Progress bars
    // ------------------------------------------------------------------
    println!("{}", title.render("Progress bars"));

    println!("{}", ProgressBar::new(0.0).label("build").render());
    println!("{}", ProgressBar::new(0.37).label("test").render());
    println!("{}", ProgressBar::new(0.82).label("package").render());
    println!("{}", ProgressBar::new(1.0).label("done").render());

    println!();
    println!(
        "{}",
        ProgressBar::new(0.64)
            .label("download")
            .width(30)
            .fill_char('=')
            .empty_char(' ')
            .fill_style(Style::new().foreground(Color("42".to_string())))
            .render()
    );
}
