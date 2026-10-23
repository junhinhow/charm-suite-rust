//! Shows how an application adapts its output to the accessibility profile.
//!
//! ```text
//! cargo run -p charm-a11y --example charm_a11y_demo
//! NO_COLOR=1 cargo run -p charm-a11y --example charm_a11y_demo
//! TERM=dumb cargo run -p charm-a11y --example charm_a11y_demo
//! A11Y_REDUCED_MOTION=1 cargo run -p charm-a11y --example charm_a11y_demo
//! ```

use charm_a11y::Profile;
use lipgloss::{rounded_border, Style};

fn main() {
    let profile = Profile::default_profile();

    let title = profile.style(
        Style::new()
            .bold(true)
            .foreground("#7C3AED")
            .border(rounded_border()),
    );
    println!(
        "{}",
        title.render(&format!("{} pronto", profile.symbol("✓", "ok")))
    );

    let frames = profile.frames(&["⠋", "⠙", "⠹", "⠸"]);
    println!("frames: {}", frames.join(" "));

    let message = format!("\u{1b}[1m{}\u{1b}[0m\n{}", "build", "concluído");
    println!("{}", profile.announce(&message));

    println!("{}", profile.describe());
}
