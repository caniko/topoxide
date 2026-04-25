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

#[test]
fn cli_validates_fixture() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_topoxide"))
        .args(["validate", "--input", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "OK");
}

#[test]
fn cli_writes_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let output = tmp.path().join("topoxide.schema.json");

    let status = Command::new(env!("CARGO_BIN_EXE_topoxide"))
        .args(["schema", "--output", output.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    let schema = std::fs::read_to_string(output).unwrap();
    assert!(schema.contains("Topoxide GraphSpec"));
    assert!(schema.contains("\"$defs\""));
    assert!(schema.contains("\"node\""));
}
