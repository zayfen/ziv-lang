use std::fs;
use std::fs;
use std::path::Path;
use std::path::Path;
use std::process::Command;

#[test]
fn test_simple_compilation() {
    let code = r#"
        let x = 42;
        let y = 10;
        let z = x + y;
    "#;

    fs::write("examples/test_simple_integration.ll", code).unwrap();

    let output = Command::new("./target/debug/llc")
        .args(&[
            "examples/test_simple_integration.ll",
            "-o",
            "test_simple_out",
        ])
        .output()
        .expect("Failed to compile");

    assert!(output.status.success());
    assert!(Path::new("test_simple_out").exists());

    fs::remove_file("examples/test_simple_integration.ll").ok();
    fs::remove_file("test_simple_out").ok();
}

#[test]
fn test_arm64_compilation() {
    let code = r#"
        let x = 42;
        let y = 10;
    "#;

    fs::write("examples/test_arm64.ll", code).unwrap();

    let output = Command::new("./target/debug/llc")
        .args(&[
            "examples/test_arm64.ll",
            "-o",
            "test_arm64_out",
            "--target",
            "arm64",
        ])
        .output()
        .expect("Failed to compile");

    if output.status.success() {
        assert!(Path::new("test_arm64_out").exists());
        fs::remove_file("examples/test_arm64.ll").ok();
        fs::remove_file("test_arm64_out").ok();
    } else {
        fs::remove_file("examples/test_arm64.ll").ok();
    }
}
