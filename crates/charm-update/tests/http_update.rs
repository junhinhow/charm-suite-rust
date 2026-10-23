#![allow(missing_docs)]

use charm_update::{Checker, Options, Updater};
use sha2::{Digest, Sha256};
use std::fs;
use std::thread;

#[test]
fn github_lookup_and_checked_install_over_loopback() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let base = format!("http://{address}");
    let payload = b"new executable".to_vec();
    let digest = format!("{:x}", Sha256::digest(&payload));
    let worker_base = base.clone();
    let worker_payload = payload.clone();
    let worker_digest = digest.clone();
    let worker = thread::spawn(move || {
        for request in server.incoming_requests().take(2) {
            if request.url().ends_with("/releases/latest") {
                let body = format!(
                    r#"{{"tag_name":"v2.0.0","body":"notes","assets":[{{"name":"app_linux_amd64","browser_download_url":"{worker_base}/asset","size":{},"digest":"sha256:{worker_digest}"}}]}}"#,
                    worker_payload.len()
                );
                request
                    .respond(tiny_http::Response::from_string(body))
                    .unwrap();
            } else {
                request
                    .respond(tiny_http::Response::from_data(worker_payload.clone()))
                    .unwrap();
            }
        }
    });

    let (release, newer) =
        Checker::with_options("owner", "repo", Options::default().with_base_url(&base))
            .unwrap()
            .check("1.0.0")
            .unwrap();
    assert!(newer);
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("app");
    fs::write(&target, b"old executable").unwrap();
    let updater = Updater::with_options(
        Options::default()
            .with_base_url(&base)
            .with_binary_name(&target),
    )
    .unwrap();
    updater.apply(&release, &release.assets[0]).unwrap();
    assert_eq!(fs::read(target).unwrap(), payload);
    worker.join().unwrap();
}
