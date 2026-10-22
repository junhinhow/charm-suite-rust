//! Prints every built-in theme with a sample of its styles.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-themes --example demo
//! ```

use charm_themes::themes;

fn main() {
    for (name, theme) in themes::themes() {
        let styles = theme.lipgloss();

        println!("{}", styles.title.render(name));
        println!(
            "{}",
            styles.subtitle.render(&format!(
                "background {} · foreground {} · primary {} · accent {}",
                theme.background.to_hex(),
                theme.foreground.to_hex(),
                theme.primary.to_hex(),
                theme.accent.to_hex()
            ))
        );
        println!();
        println!("{}", styles.base.render("Base text"));
        println!("{}", styles.bold.render("Bold text"));
        println!("{}", styles.italic.render("Italic text"));
        println!("{}", styles.faint.render("Faint text"));
        println!();
        println!("{}", styles.success.render("Success"));
        println!("{}", styles.warning.render("Warning"));
        println!("{}", styles.error.render("Error"));
        println!("{}", styles.info.render("Info"));
        println!();
        println!("{}", styles.highlight.render("Highlight"));
        println!("{}", styles.code.render("let x = 42;"));
        println!();
        println!("{}", styles.border.render("Boxed content"));
        println!();
    }
}
