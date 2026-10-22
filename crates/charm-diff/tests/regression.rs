//! Regression cases ported from the Go `regression_test.go` suite.

use charm_core::{display_width, strip_ansi};
use charm_diff::{side_by_side_diff, unified_diff_with_options, Options};

#[test]
fn crlf_input_is_normalized_before_rendering() {
    let crlf = unified_diff_with_options("a\r\nb\r\nc\r\n", "a\r\nX\r\nc\r\n", &Options::default());
    let lf = unified_diff_with_options("a\nb\nc\n", "a\nX\nc\n", &Options::default());
    assert!(!crlf.contains('\r'));
    assert_eq!(crlf, lf);
}

#[test]
fn side_by_side_respects_requested_width_for_short_and_long_lines() {
    for (width, old, new) in [
        (80, "hello\n", "world\n"),
        (
            80,
            "uma linha bem mais longa que o espaco disponivel\n",
            "outra\n",
        ),
        (100, &"a\n".repeat(30), &"b\n".repeat(30)),
        (60, "x\n", "y\n"),
        (40, "x\n", "y\n"),
    ] {
        let output = side_by_side_diff(old, new, width);
        for line in output.lines() {
            assert!(display_width(line) <= width, "width {width}: {line:?}");
        }
    }
}

#[test]
fn side_by_side_width_accounts_for_multibyte_display_cells() {
    let output = side_by_side_diff(
        "acentuação e emoji 🎉 aqui\n",
        "日本語のテキストと_more_\n",
        80,
    );
    for line in output.lines() {
        assert!(display_width(line) <= 80, "wide line: {line:?}");
    }
}

#[test]
fn truncation_never_exceeds_width_or_splits_utf8() {
    let cases = [
        "áéíóú çãõ",
        "日本語のテキスト",
        "emoji 🎉🎊 aqui",
        "mixto á日🎉",
    ];
    for text in cases {
        let output = side_by_side_diff(&format!("{text}\n"), "replacement\n", 21);
        for line in output.lines() {
            assert!(display_width(line) <= 21, "wide line: {line:?}");
            assert!(std::str::from_utf8(line.as_bytes()).is_ok());
        }
    }
}

#[test]
fn line_number_width_grows_for_files_with_ten_thousand_lines() {
    let mut old = (0..10_000)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>();
    let mut new = old.clone();
    old[9_999] = "old final line".to_owned();
    new[9_999] = "new final line".to_owned();
    let old = format!("{}\n", old.join("\n"));
    let new = format!("{}\n", new.join("\n"));
    let output = strip_ansi(&unified_diff_with_options(
        &old,
        &new,
        &Options {
            context_lines: 0,
            ..Options::default()
        },
    ));
    assert!(output.contains("10000       -old final line"), "{output:?}");
    assert!(output.contains("      10000 +new final line"), "{output:?}");
}

#[test]
fn ten_thousand_line_diff_uses_the_default_linear_space_engine() {
    let mut lines = (0..10_000)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>();
    let old = format!("{}\n", lines.join("\n"));
    lines[5_000] = "line altered".to_owned();
    lines[9_999] = "last line altered".to_owned();
    let new = format!("{}\n", lines.join("\n"));
    let output = strip_ansi(&unified_diff_with_options(
        &old,
        &new,
        &Options {
            context_lines: 3,
            ..Options::default()
        },
    ));
    assert!(output.contains("-line 5000"));
    assert!(output.contains("+line altered"));
    assert!(output.contains("-line 9999"));
    assert!(output.contains("+last line altered"));
}

#[test]
fn zero_context_is_the_unsigned_rust_equivalent_of_negative_go_context() {
    let output = strip_ansi(&unified_diff_with_options(
        "a\nb\nc\n",
        "a\nX\nc\n",
        &Options {
            context_lines: 0,
            ..Options::default()
        },
    ));
    assert!(output.contains("-b\n"));
    assert!(output.contains("+X\n"));
    assert!(output.starts_with("@@ -2 +2 @@"));
}
