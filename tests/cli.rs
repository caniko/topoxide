use std::process::Command;

#[test]
fn cli_renders_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    let output = tmp.path().join("simple.svg");

    let status = Command::new(env!("CARGO_BIN_EXE_topoxide"))
        .args([
            "render",
            "--input",
            "tests/fixtures/simple.toml",
            "--output",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("Simple graph"));
    assert!(svg.contains("Dataset"));
    assert!(svg.contains("neurons"));
}
