use std::process::Command;

#[test]
fn cli_renders_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    let output = tmp.path().join("simple.svg");
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple.toml");

    let status = Command::new(env!("CARGO_BIN_EXE_topoxide"))
        .args([
            "render",
            "--input",
            input.to_str().unwrap(),
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
