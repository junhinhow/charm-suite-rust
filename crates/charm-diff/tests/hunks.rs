//! Regression coverage for Git-compatible unified hunk construction.

use charm_diff::{unified_diff_with_options, Options};

fn plain_diff(old: &str, new: &str, context_lines: usize) -> String {
    charm_core::strip_ansi(&unified_diff_with_options(
        old,
        new,
        &Options {
            context_lines,
            ..Options::default()
        },
    ))
}

#[test]
fn insertion_led_overlapping_windows_merge_into_one_hunk() {
    let output = plain_diff("o00\no01\no02\no03\n", "n00\nn01\no02\nn03\n", 3);
    assert_eq!(output.matches("@@ -").count(), 1);
    assert!(output.starts_with("@@ -1,4 +1,4 @@\n"));
}

#[test]
fn insertion_at_start_does_not_create_overlapping_hunks() {
    let output = plain_diff("a\nb\nc\nd\n", "X\na\nb\nc\nd\n", 3);
    assert_eq!(output.matches("@@ -").count(), 1);
}

#[test]
fn insertion_only_hunk_ranges_match_git_conventions() {
    for (old, new, expected) in [
        ("", "a\nb\n", "@@ -0,0 +1,2 @@"),
        ("a\n", "x\na\n", "@@ -1 +1,2 @@"),
        ("a\nb\nc\n", "a\nx\nb\nc\n", "@@ -1,3 +1,4 @@"),
        ("a\nb\nc\n", "a\nb\nc\nd\n", "@@ -1,3 +1,4 @@"),
    ] {
        let output = plain_diff(old, new, 3);
        assert!(
            output.starts_with(expected),
            "expected {expected:?}: {output:?}"
        );
    }
}

#[test]
fn distant_changes_are_split_when_context_does_not_reach() {
    let old = (0..20).map(|i| format!("line {i}")).collect::<Vec<_>>();
    let mut new = old.clone();
    new[1] = "CHANGED".to_owned();
    new[15] = "CHANGED TOO".to_owned();
    let output = plain_diff(
        &format!("{}\n", old.join("\n")),
        &format!("{}\n", new.join("\n")),
        3,
    );
    assert_eq!(output.matches("@@ -").count(), 2);
}

#[test]
fn zero_context_keeps_changes_without_context_lines() {
    let output = plain_diff("a\nb\nc\n", "a\nX\nc\n", 0);
    assert!(output.starts_with("@@ -2 +2 @@\n"));
    assert!(output.contains("-b\n"));
    assert!(output.contains("+X\n"));
    assert!(!output.contains(" a\n"));
}
