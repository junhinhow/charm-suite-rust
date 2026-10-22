#![no_main]

use charm_banner::{render, Font, RenderOption};
use libfuzzer_sys::fuzz_target;

const MAX_CHARS: usize = 200;

fuzz_target!(|text: &str| {
    let text: String = text.chars().take(MAX_CHARS).collect();

    for font in [Font::Small, Font::Medium, Font::Large] {
        let _ = render(&text, &[RenderOption::Font(font)]);
    }
});
