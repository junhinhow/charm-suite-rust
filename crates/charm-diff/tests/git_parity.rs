//! Differential tests against `git diff`, ported from Go's golden tests.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use charm_core::strip_ansi;
use charm_diff::{unified_diff_with_options, Options};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TemporaryFiles(PathBuf);

impl TemporaryFiles {
    fn new(old: &str, new: &str) -> Self {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("charm-diff-git-parity-{}-{id}", std::process::id()));
        fs::create_dir(&dir).expect("create temporary git-parity directory");
        fs::write(dir.join("old.txt"), old).expect("write old test input");
        fs::write(dir.join("new.txt"), new).expect("write new test input");
        Self(dir)
    }

    fn old_path(&self) -> PathBuf {
        self.0.join("old.txt")
    }

    fn new_path(&self) -> PathBuf {
        self.0.join("new.txt")
    }
}

impl Drop for TemporaryFiles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git_diff_lines(old: &str, new: &str, context: usize) -> Vec<String> {
    let files = TemporaryFiles::new(old, new);
    let output = Command::new("git")
        .args([
            "-c",
            "core.autocrlf=false",
            "diff",
            "--no-index",
            "--no-color",
        ])
        .arg(format!("--unified={context}"))
        .arg("--")
        .arg(files.old_path())
        .arg(files.new_path())
        .output()
        .expect("run git diff");
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "git diff failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("git output should be UTF-8");
    let lines = text.lines().collect::<Vec<_>>();
    let Some(first_hunk) = lines.iter().position(|line| line.starts_with("@@")) else {
        return Vec::new();
    };
    lines[first_hunk..]
        .iter()
        .map(|line| hunk_header_only(line).to_owned())
        .collect()
}

fn hunk_header_only(line: &str) -> &str {
    if !line.starts_with("@@") {
        return line;
    }
    let rest = &line[2..];
    match rest.find("@@") {
        Some(index) => &line[..index + 4],
        None => line,
    }
}

fn rust_diff_lines(old: &str, new: &str, context: usize) -> Vec<String> {
    let options = Options {
        context_lines: context,
        ..Options::default()
    };
    let output = strip_ansi(&unified_diff_with_options(old, new, &options));
    if output.is_empty() {
        return Vec::new();
    }
    let width = line_number_width(old.lines().count(), new.lines().count());
    let prefix_width = 2 * width + 2;
    output
        .lines()
        .map(|line| {
            if line.starts_with("@@") {
                hunk_header_only(line).to_owned()
            } else {
                line.get(prefix_width..).unwrap_or(line).to_owned()
            }
        })
        .collect()
}

fn line_number_width(old_count: usize, new_count: usize) -> usize {
    let mut value = old_count.max(new_count);
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits.max(4)
}

#[test]
fn randomized_unified_diffs_match_git_xdiff() {
    if Command::new("git").arg("--version").output().is_err() {
        eprintln!("git is unavailable; skipping xdiff differential test");
        return;
    }

    let alphabet = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let mut random = Xorshift64(42);
    for case in 0..200 {
        let old_len = 1 + (random.next() % 40) as usize;
        let old = (0..old_len)
            .map(|_| alphabet[(random.next() % alphabet.len() as u64) as usize])
            .collect::<Vec<_>>();
        let mut new = old.clone();
        let mutations = 1 + (random.next() % 6) as usize;
        for _ in 0..mutations {
            match random.next() % 3 {
                0 if !new.is_empty() => {
                    let index = (random.next() % new.len() as u64) as usize;
                    new.remove(index);
                }
                1 => {
                    let index = (random.next() % (new.len() + 1) as u64) as usize;
                    let value = alphabet[(random.next() % alphabet.len() as u64) as usize];
                    new.insert(index, value);
                }
                _ if !new.is_empty() => {
                    let index = (random.next() % new.len() as u64) as usize;
                    new[index] = alphabet[(random.next() % alphabet.len() as u64) as usize];
                }
                _ => {}
            }
        }
        let old = format!("{}\n", old.join("\n"));
        let new = if new.is_empty() {
            String::new()
        } else {
            format!("{}\n", new.join("\n"))
        };
        let context = (random.next() % 4) as usize;
        let actual = rust_diff_lines(&old, &new, context);
        let expected = git_diff_lines(&old, &new, context);
        assert_eq!(
            actual, expected,
            "random case {case}, context={context}\nold={old:?}\nnew={new:?}"
        );
    }
}

struct Xorshift64(u64);

impl Xorshift64 {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
