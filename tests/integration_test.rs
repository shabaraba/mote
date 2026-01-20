use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

struct TestContext {
    _temp_dir: TempDir,
    project_dir: PathBuf,
    mote_bin: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_dir = temp_dir.path().to_path_buf();

        let mote_bin = std::env::current_exe()
            .expect("Failed to get current executable path")
            .parent()
            .expect("Failed to get parent directory")
            .parent()
            .expect("Failed to get grandparent directory")
            .join("mote");

        Self {
            _temp_dir: temp_dir,
            project_dir,
            mote_bin,
        }
    }

    fn run_mote(&self, args: &[&str]) -> std::process::Output {
        Command::new(&self.mote_bin)
            .args(args)
            .current_dir(&self.project_dir)
            .output()
            .expect("Failed to execute mote")
    }

    fn write_file(&self, path: &str, content: &str) {
        let file_path = self.project_dir.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        fs::write(file_path, content).expect("Failed to write file");
    }

    fn read_file(&self, path: &str) -> String {
        let file_path = self.project_dir.join(path);
        fs::read_to_string(file_path).expect("Failed to read file")
    }

    fn file_exists(&self, path: &str) -> bool {
        self.project_dir.join(path).exists()
    }
}

#[test]
fn test_init_creates_directory_structure() {
    let ctx = TestContext::new();

    let output = ctx.run_mote(&["init"]);
    assert!(output.status.success());

    assert!(ctx.file_exists(".mote"));
    assert!(ctx.file_exists(".moteignore"));
    assert!(ctx.file_exists(".mote/objects"));
    assert!(ctx.file_exists(".mote/snapshots"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initialized mote"));
}

#[test]
fn test_snapshot_creates_snapshot() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test1.txt", "Hello World");
    ctx.write_file("test2.txt", "Test content");

    let output = ctx.run_mote(&["snapshot", "-m", "Initial snapshot"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Created snapshot"));
    assert!(stdout.contains("Initial snapshot"));
}

#[test]
fn test_log_shows_snapshots() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "content");
    ctx.run_mote(&["snapshot", "-m", "Test snapshot"]);

    let output = ctx.run_mote(&["log"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test snapshot"));
    assert!(stdout.contains("snapshot"));
}

#[test]
fn test_log_oneline_format() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "content");
    ctx.run_mote(&["snapshot", "-m", "Test message"]);

    let output = ctx.run_mote(&["log", "--oneline"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(lines.len() > 0);
    assert!(lines[0].contains("Test message"));
    assert!(lines[0].contains("files"));
}

#[test]
fn test_diff_detects_changes() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "original");
    let output1 = ctx.run_mote(&["snapshot", "-m", "First"]);
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let snapshot_id: String = stdout1
        .split_whitespace()
        .find(|s| s.len() == 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .expect("Could not find snapshot ID")
        .to_string();

    ctx.write_file("test.txt", "modified");

    let output = ctx.run_mote(&["diff", &snapshot_id]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diff --mote a/test.txt b/test.txt"));
}

#[test]
fn test_diff_with_content() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "Line 1");
    let output1 = ctx.run_mote(&["snapshot", "-m", "First"]);
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let snapshot_id1: String = stdout1
        .split_whitespace()
        .find(|s| s.len() == 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap()
        .to_string();

    ctx.write_file("test.txt", "Line 2");
    let output2 = ctx.run_mote(&["snapshot", "-m", "Second"]);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let snapshot_id2: String = stdout2
        .split_whitespace()
        .find(|s| s.len() == 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap()
        .to_string();

    let output = ctx.run_mote(&["diff", &snapshot_id1, &snapshot_id2]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diff --mote a/test.txt b/test.txt"));
    assert!(stdout.contains("Line 1") || stdout.contains("Line 2"));
}

#[test]
fn test_restore_single_file() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "original content");
    let output1 = ctx.run_mote(&["snapshot", "-m", "Original"]);
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let snapshot_id: String = stdout1
        .split_whitespace()
        .find(|s| s.len() == 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap()
        .to_string();

    ctx.write_file("test.txt", "modified content");

    let output = ctx.run_mote(&["restore", &snapshot_id, "--file", "test.txt"]);
    assert!(output.status.success());

    let content = ctx.read_file("test.txt");
    assert_eq!(content, "original content");
}

#[test]
fn test_restore_dry_run() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "original");
    let output1 = ctx.run_mote(&["snapshot", "-m", "Original"]);
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let snapshot_id: String = stdout1
        .split_whitespace()
        .find(|s| s.len() == 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap()
        .to_string();

    ctx.write_file("test.txt", "modified");
    let modified_content = ctx.read_file("test.txt");

    let output = ctx.run_mote(&["restore", &snapshot_id, "--dry-run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dry-run"));
    assert!(stdout.contains("Would restore"));

    let content_after = ctx.read_file("test.txt");
    assert_eq!(content_after, modified_content);
}

#[test]
fn test_auto_mode_skip_if_no_changes() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "content");
    ctx.run_mote(&["snapshot", "--auto"]);

    ctx.run_mote(&["snapshot", "--auto"]);

    let output = ctx.run_mote(&["log"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let snapshot_count = stdout.matches("snapshot").count();

    assert_eq!(snapshot_count, 1);
}

#[test]
fn test_auto_mode_create_if_changes() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "version 1");
    ctx.run_mote(&["snapshot", "--auto"]);

    ctx.write_file("test.txt", "version 2");
    ctx.run_mote(&["snapshot", "--auto"]);

    let output = ctx.run_mote(&["log"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let snapshot_count = stdout.matches("snapshot").count();

    assert_eq!(snapshot_count, 2);
}

#[test]
fn test_show_displays_snapshot_details() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    ctx.write_file("test.txt", "content");
    let output1 = ctx.run_mote(&["snapshot", "-m", "Test snapshot"]);
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let snapshot_id: String = stdout1
        .split_whitespace()
        .find(|s| s.len() == 7 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap()
        .to_string();

    let output = ctx.run_mote(&["show", &snapshot_id]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test snapshot"));
    assert!(stdout.contains("test.txt"));
    assert!(stdout.contains("Files:"));
}

#[test]
fn test_snapshot_without_init_fails() {
    let ctx = TestContext::new();

    ctx.write_file("test.txt", "content");
    let output = ctx.run_mote(&["snapshot"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn test_empty_project_snapshot() {
    let ctx = TestContext::new();
    ctx.run_mote(&["init"]);

    let output = ctx.run_mote(&["snapshot"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No files to snapshot") || stdout.contains("Created snapshot"));
}
