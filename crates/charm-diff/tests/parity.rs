//! Parity tests against the Go implementation (`charm-diff`).
//!
//! The golden file `tests/golden/diff.txt` is produced by the Go program
//! `reference/go/charm-diff/cmd/parity`:
//!
//! ```text
//! cargo run -p charm-parity --bin parity -- gen crates/charm-diff/tests/golden/diff.txt -- go run ./reference/go/charm-diff/cmd/parity
//! ```
//!
//! The list of cases below must stay in sync with the one used by the Go
//! program.

use std::fmt::Write as _;

use charm_diff::{side_by_side_diff_with_options, unified_diff_with_options, DiffStyle, Options};
use charm_parity::assert_golden;

/// A sample rendered by both implementations.
struct TestCase {
    /// Name of the case, used as the section header.
    name: &'static str,
    /// Original text.
    old: &'static str,
    /// New text.
    new: &'static str,
    /// Number of context lines.
    context_lines: usize,
    /// Whether the GitHub style must be used instead of the default one.
    github_style: bool,
}

/// Total widths used for the side-by-side rendering.
const WIDTHS: [usize; 3] = [40, 80, 120];

/// The samples rendered by both implementations.
const CASES: &[TestCase] = &[
    TestCase {
        name: "identical",
        old: "alpha\nbravo\ncharlie\ndelta\n",
        new: "alpha\nbravo\ncharlie\ndelta\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "insertion",
        old: "alpha\nbravo\ncharlie\ndelta\n",
        new: "alpha\nbravo\ninserted\ncharlie\ndelta\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "deletion",
        old: "alpha\nbravo\ncharlie\ndelta\n",
        new: "alpha\ncharlie\ndelta\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "modification",
        old: "alpha\nbravo\ncharlie\ndelta\n",
        new: "alpha\nbravo modified\ncharlie\ndelta\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "adjacent_changes",
        old: "one\ntwo\nthree\nfour\nfive\n",
        new: "one\nTWO\nTHREE\nfour\nFIVE\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "distant_changes",
        old: "l01\nl02\nl03\nl04\nl05\nl06\nl07\nl08\nl09\nl10\nl11\nl12\n",
        new: "l01\nl02\nL03\nl04\nl05\nl06\nl07\nl08\nL09\nl10\nl11\nl12\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "empty_to_content",
        old: "",
        new: "first\nsecond\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "content_to_empty",
        old: "first\nsecond\n",
        new: "",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "no_trailing_newline",
        old: "alpha\nbravo\ncharlie",
        new: "alpha\nbravo\nCHANGED",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "unicode_accents",
        old: "acentuação\n日本語のテキスト\nemoji 🎉\n",
        new: "acentuação corrigida\n日本語のテキスト\nemoji 🎉\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "blank_lines",
        old: "a\n\n\nb\n\n",
        new: "a\n\n\nB\n\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "context_0",
        old: "l1\nl2\nl3\nl4\nl5\n",
        new: "l1\nl2\nl3 changed\nl4\nl5\n",
        context_lines: 0,
        github_style: false,
    },
    TestCase {
        name: "context_1",
        old: "l1\nl2\nl3\nl4\nl5\n",
        new: "l1\nl2\nl3 changed\nl4\nl5\n",
        context_lines: 1,
        github_style: false,
    },
    TestCase {
        name: "context_3",
        old: "l1\nl2\nl3\nl4\nl5\n",
        new: "l1\nl2\nl3 changed\nl4\nl5\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "context_10",
        old: "l1\nl2\nl3\nl4\nl5\n",
        new: "l1\nl2\nl3 changed\nl4\nl5\n",
        context_lines: 10,
        github_style: false,
    },
    TestCase {
        name: "tabs",
        old: "func main() {\n\tprintln!(\"hi\");\n}\n",
        new: "func main() {\n\tprintln!(\"hello\");\n}\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "long_lines",
        old: "short\nthis is a very long line that will not fit in a narrow column\n",
        new: "short\nthis is a very long line that will not fit in a narrow column either\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "repeated_lines",
        old: "x\nx\nx\nx\n",
        new: "x\nx\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "full_rewrite",
        old: "one\ntwo\nthree\n",
        new: "completely\ndifferent\ncontent\n",
        context_lines: 3,
        github_style: false,
    },
    TestCase {
        name: "github_style",
        old: "alpha\nbravo\ncharlie\ndelta\n",
        new: "alpha\nbravo modified\ncharlie\ndelta\n",
        context_lines: 3,
        github_style: true,
    },
];

/// Renders every sample, in the same order as the Go program.
fn render_report() -> String {
    let mut out = String::new();
    for case in CASES {
        let style = if case.github_style {
            DiffStyle::github()
        } else {
            DiffStyle::default()
        };
        let opts = Options {
            context_lines: case.context_lines,
            style,
            ..Options::default()
        };

        let _ = writeln!(out, "### CASE {}", case.name);
        let _ = writeln!(out, "--- unified context={} ---", case.context_lines);
        out.push_str(&unified_diff_with_options(case.old, case.new, &opts));
        for width in WIDTHS {
            let _ = writeln!(out, "--- side-by-side width={width} ---");
            out.push_str(&side_by_side_diff_with_options(
                case.old, case.new, width, &opts,
            ));
        }
        let _ = writeln!(out, "### END CASE {}", case.name);
    }
    out
}

#[test]
fn golden_matches_the_go_implementation() {
    assert_golden("diff", &render_report());
}

/// Strict counterpart of [`golden_matches_the_go_implementation`]: the ANSI
/// codes are removed from both sides but *nothing else* is normalized, so the
/// column padding is compared byte by byte as well.
#[test]
fn golden_matches_the_go_implementation_byte_for_byte() {
    let expected = std::fs::read_to_string(charm_parity::golden_path("diff"))
        .expect("falha ao ler o golden file `diff`");
    let actual = render_report();

    assert_eq!(
        charm_core::strip_ansi(&expected),
        charm_core::strip_ansi(&actual),
        "a saida do Rust diverge da saida do Go"
    );
}
