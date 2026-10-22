#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

const MAX_BYTES: usize = 4 * 1024;

#[derive(Arbitrary, Debug)]
struct Input {
    old: String,
    new: String,
    width: u8,
}

fn clip(s: &str) -> &str {
    if s.len() <= MAX_BYTES {
        return s;
    }
    let mut end = MAX_BYTES;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn assert_utf8(out: &str) {
    assert!(std::str::from_utf8(out.as_bytes()).is_ok());
}

fuzz_target!(|input: Input| {
    let old = clip(&input.old);
    let new = clip(&input.new);

    assert_utf8(&charm_diff::unified_diff(old, new));
    assert_utf8(&charm_diff::side_by_side_diff(
        old,
        new,
        usize::from(input.width),
    ));
});
