#![allow(missing_docs)]

use charm_update::{choose_asset, Asset};

#[test]
fn platform_selection_matches_golden() {
    let assets = [
        Asset {
            name: "tool_linux_x86_64.zip".into(),
            ..Default::default()
        },
        Asset {
            name: "tool_linux_amd64.tar.gz".into(),
            ..Default::default()
        },
        Asset {
            name: "tool_linux_amd64.tar.gz.sha256".into(),
            ..Default::default()
        },
    ];
    let selected = choose_asset("linux", "amd64", &assets).unwrap();
    assert_eq!(
        selected.name,
        include_str!("golden/selected-asset.txt").trim()
    );
}
