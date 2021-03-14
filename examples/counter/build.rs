use std::fs::rename;
use std::process::Command;

fn main() {
    let target = "wasm32-unknown-unknown";
    let opt_level = "-O1";
    let crate_name = "client";
    let out_dir = format!("static-{}", &crate_name);
    let temp_file = "_tmp.wasm";
    let wasm_file = format!("{}/{}_bg.wasm", &out_dir, &crate_name);

    // Build client as WASM.
    Command::new("cargo")
        .args(&[
            "build",
            "-p",
            crate_name,
            "--lib",
            "--release",
            "--target",
            target,
        ])
        .status()
        .expect("Failed to compile client as wasm32.");

    // Wasm-Bindgen
    Command::new("wasm-bindgen")
        .args(&[
            &format!("target/{}/release/{}.wasm", target, crate_name),
            "--out-dir",
            &out_dir,
            "--target",
            "web",
        ])
        .status()
        .expect("Failed to run wasm-bindgen.");

    // Wasm-opt
    Command::new("wasm-opt")
        .args(&[opt_level, "-o", temp_file, &wasm_file])
        .status()
        .expect("Failed to run wasm-opt.");

    rename(&temp_file, &wasm_file).expect(&format!(
        "Failed to replace {} with {}",
        &wasm_file, &temp_file
    ));
}
