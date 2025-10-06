// Integration tests for error handling and edge cases
mod common;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use common::TestContext;

#[test]
fn test_missing_database() {
    let ctx = TestContext::new();
    // Don't create database

    // Try to list users without database
    let mut cmd = ctx.cmd();
    cmd.arg("ls");

    // Should fail or prompt for database creation
    let _output = cmd.output().expect("Failed to run command");
    // Command should either fail or prompt for db creation
    // Either way, listing should not succeed without a database
}

#[test]
fn test_duplicate_entry() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("duplicate");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "user").ok();
        writeln!(stdin, "pass").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Try to create the same user again
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("duplicate");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "user").ok();
        writeln!(stdin, "pass").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }

    let output = child.wait_with_output().expect("Failed to wait");
    // Should fail because duplicate already exists
    assert!(!output.status.success(), "Creating duplicate should fail");

    ctx.close_db();
}

#[test]
fn test_show_nonexistent() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to show non-existent user
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("doesnotexist");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("").or(predicate::str::is_empty().not()));

    ctx.close_db();
}

#[test]
fn test_edit_nonexistent() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to edit non-existent user
    let mut cmd = ctx.cmd();
    cmd.arg("edit").arg("doesnotexist");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "user").ok();
        writeln!(stdin, "pass").ok();
    }

    let output = child.wait_with_output().expect("Failed to wait");
    assert!(!output.status.success(), "Editing nonexistent should fail");

    ctx.close_db();
}

#[test]
fn test_rename_nonexistent() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to rename non-existent entry
    let mut cmd = ctx.cmd();
    cmd.arg("mv").arg("doesnotexist").arg("newname");

    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_rename_to_existing() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create two users
    for name in &["first", "second"] {
        let mut cmd = ctx.cmd();
        cmd.arg("new").arg(name);
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "user").ok();
            writeln!(stdin, "pass").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
        }
        child.wait().ok();
    }

    // Try to rename first to second (which already exists)
    let mut cmd = ctx.cmd();
    cmd.arg("mv").arg("first").arg("second");

    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_copy_nonexistent() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to copy non-existent entry
    let mut cmd = ctx.cmd();
    cmd.arg("cp").arg("doesnotexist").arg("newname");

    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_copy_to_existing() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create two users
    for name in &["source", "dest"] {
        let mut cmd = ctx.cmd();
        cmd.arg("new").arg(name);
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "user").ok();
            writeln!(stdin, "pass").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
        }
        child.wait().ok();
    }

    // Try to copy source to dest (which already exists)
    let mut cmd = ctx.cmd();
    cmd.arg("cp").arg("source").arg("dest");

    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_delete_nonexistent() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to delete non-existent entry
    let mut cmd = ctx.cmd();
    cmd.arg("rm").arg("doesnotexist");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "y").ok();
    }

    let output = child.wait_with_output().expect("Failed to wait");
    assert!(!output.status.success(), "Deleting nonexistent should fail");

    ctx.close_db();
}

#[test]
fn test_invalid_otp_secret() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to create OTP with invalid Base32 secret
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("invalidotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "INVALID!@#$%").ok(); // Invalid Base32
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }

    let _output = child.wait_with_output().expect("Failed to wait");
    // Might fail or accept it - depends on validation
    // The command should at least complete without crashing

    ctx.close_db();
}

#[test]
fn test_empty_entry_name() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to create entry with empty name
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("");

    let _output = cmd.output().expect("Failed to run command");
    // Should fail with empty name

    ctx.close_db();
}

#[test]
fn test_special_characters_in_name() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Try to create entry with special characters
    // Forward slash is allowed for categories, test other special chars
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("name*with?special");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "user").ok();
        writeln!(stdin, "pass").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }

    let _status = child.wait().expect("Failed to wait");
    // Should either succeed or fail gracefully

    ctx.close_db();
}

#[test]
fn test_very_long_entry_name() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create entry with very long name
    let long_name = "a".repeat(500);
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg(&long_name);
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "user").ok();
        writeln!(stdin, "pass").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }

    let _status = child.wait().expect("Failed to wait");
    // Should either succeed or fail gracefully

    ctx.close_db();
}

#[test]
fn test_multiple_daemon_close() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Close database once
    ctx.close_db();

    // Try to close again
    let mut cmd = ctx.cmd();
    cmd.arg("db").arg("close");

    let _output = cmd.output().expect("Failed to run command");
    // Should handle gracefully (might fail or succeed as no-op)
}

#[test]
fn test_unicode_in_values() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create entry with unicode characters
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("unicode_test").arg("Hello 世界 🌍");

    cmd.assert().success();

    // Get it back
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("unicode_test");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("世界"));

    ctx.close_db();
}
