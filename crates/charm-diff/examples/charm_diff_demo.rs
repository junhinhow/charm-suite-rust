//! Demo of the `charm-diff` crate.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-diff --example demo
//! ```

use charm_diff::{side_by_side_diff, unified_diff, unified_diff_with_options, DiffStyle, Options};

fn main() {
    let old = "\
fn main() {
    println!(\"hello\");
}
";
    let new = "\
fn main() {
    println!(\"hello, world!\");
    println!(\"welcome to the diff\");
}
";

    println!("=== unified diff (default options) ===");
    print!("{}", unified_diff(old, new));

    println!();
    println!("=== unified diff (10 context lines, GitHub style) ===");
    let opts = Options {
        context_lines: 10,
        style: DiffStyle::github(),
    };
    print!("{}", unified_diff_with_options(old, new, &opts));

    println!();
    println!("=== side-by-side diff (100 columns) ===");
    print!("{}", side_by_side_diff(old, new, 100));
}
