use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../frontend/src");

    let status = Command::new("wasm-pack")
        .args(&[
            "build",
            "--release",
            "--target",
            "web",
            "--out-name",
            "wasm",
            "--out-dir",
            "static",
        ])
        .current_dir("../frontend")
        .status()
        .expect("Failed to run wasm-pack");
    assert!(status.success(), "Nonzero exit code of wasm-pack");
}
