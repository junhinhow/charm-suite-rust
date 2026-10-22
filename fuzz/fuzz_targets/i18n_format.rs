#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::{json, Value};

const MAX_TEMPLATE_BYTES: usize = 1024;

fn args() -> Vec<Value> {
    vec![
        json!("Alice"),
        json!(5),
        json!(-7),
        json!(3.5),
        json!(true),
        json!(null),
        json!([1, 2, 3]),
        json!({ "key": "value" }),
    ]
}

fuzz_target!(|template: &str| {
    let mut end = template.len().min(MAX_TEMPLATE_BYTES);
    while !template.is_char_boundary(end) {
        end -= 1;
    }
    let template = &template[..end];

    let _ = charm_i18n::format::sprintf(template, &args());
    let _ = charm_i18n::format::sprintf(template, &[]);
});
