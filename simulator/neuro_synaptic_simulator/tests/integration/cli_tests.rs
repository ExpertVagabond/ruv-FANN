use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test basic CLI invocation
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("RUV FANN Simulator"));
}

/// Test version flag
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test running a simple WASM module
#[test]
fn test_run_simple_wasm() {
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/add.wasm");
    
    cmd.arg("run")
        .arg(&wasm_path)
        .arg("--inputs").arg("2.5,3.5")
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 6.0"));
}

/// Test 256-core parallel execution
#[test]
fn test_parallel_execution() {
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/multiply.wasm");
    
    cmd.arg("run")
        .arg(&wasm_path)
        .arg("--cores").arg("256")
        .arg("--inputs").arg("4.0,5.0")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Using 256 cores"))
        .stdout(predicate::str::contains("Result: 20.0"));
}

/// Test custom configuration file
#[test]
fn test_custom_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    fs::write(&config_path, r#"
[simulator]
cores = 128
memory_limit = 512
enable_visualization = false

[neural_network]
layer_sizes = [10, 20, 10]
activation = "relu"
"#).unwrap();
    
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/neural_net.wasm");
    
    cmd.arg("run")
        .arg(&wasm_path)
        .arg("--config").arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Using 128 cores"));
}

/// Test batch processing multiple WASM modules
#[test]
fn test_batch_processing() {
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules");
    
    cmd.arg("batch")
        .arg(&wasm_dir)
        .arg("--pattern").arg("*.wasm")
        .arg("--inputs").arg("1.0,2.0,3.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed"))
        .stdout(predicate::str::contains("modules"));
}

/// Test error handling for invalid WASM
#[test]
fn test_invalid_wasm() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_wasm = temp_dir.path().join("invalid.wasm");
    fs::write(&invalid_wasm, b"not a valid wasm file").unwrap();
    
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    cmd.arg("run")
        .arg(&invalid_wasm)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// Test memory limit enforcement
#[test]
fn test_memory_limit() {
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/memory_intensive.wasm");
    
    cmd.arg("run")
        .arg(&wasm_path)
        .arg("--memory-limit").arg("64")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Memory limit exceeded"));
}

/// Test visualization output
#[test]
fn test_visualization_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("viz_output");
    
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/neural_net.wasm");
    
    cmd.arg("run")
        .arg(&wasm_path)
        .arg("--visualize")
        .arg("--output").arg(&output_dir)
        .assert()
        .success();
    
    // Check that visualization files were created
    assert!(output_dir.join("network_graph.html").exists());
    assert!(output_dir.join("activation_heatmap.png").exists());
}

/// Test performance profiling output
#[test]
fn test_profiling() {
    let temp_dir = TempDir::new().unwrap();
    let profile_path = temp_dir.path().join("profile.json");
    
    let mut cmd = Command::cargo_bin("ruv-fann-simulator").unwrap();
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_modules/add.wasm");
    
    cmd.arg("run")
        .arg(&wasm_path)
        .arg("--profile").arg(&profile_path)
        .arg("--inputs").arg("1.0,2.0")
        .assert()
        .success();
    
    // Verify profile file was created and contains expected data
    assert!(profile_path.exists());
    let profile_content = fs::read_to_string(&profile_path).unwrap();
    assert!(profile_content.contains("execution_time"));
    assert!(profile_content.contains("memory_usage"));
}