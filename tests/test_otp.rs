// Integration tests for OTP (OAuth) operations
mod common;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use common::TestContext;

#[test]
fn test_otp_new() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a new OTP entry with a valid Base32 secret
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("github");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn nyx");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok(); // Valid Base32 secret
        writeln!(stdin, "https://github.com").ok();
        writeln!(stdin, "recovery1 recovery2").ok();
        writeln!(stdin, "").ok();
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "OTP creation should succeed");

    ctx.close_db();
}

#[test]
fn test_otp_list() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create multiple OTP entries
    for name in &["gitlab", "bitbucket", "aws"] {
        let mut cmd = ctx.cmd();
        cmd.arg("otp").arg("new").arg(name);
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
        }
        child.wait().ok();
    }

    // List all OTP entries
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("gitlab"))
        .stdout(predicate::str::contains("bitbucket"))
        .stdout(predicate::str::contains("aws"));

    ctx.close_db();
}

#[test]
fn test_otp_generate() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create an OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("testservice");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Generate OTP code
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("testservice");

    // Should output a 6-digit code
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"\d{6}").unwrap());

    ctx.close_db();
}

#[test]
fn test_otp_show() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create an OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("showtest");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "https://test.com").ok();
        writeln!(stdin, "rec1 rec2").ok();
    }
    child.wait().ok();

    // Show the OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("showtest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("https://test.com"))
        .stdout(predicate::str::contains("rec1 rec2"));

    ctx.close_db();
}

#[test]
fn test_otp_edit() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create an OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("editotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "original.com").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Edit the OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("edit").arg("editotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "updated.com").ok();
        writeln!(stdin, "new recovery keys").ok();
    }
    child.wait().ok();

    // Verify changes
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("editotp");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("updated.com"));

    ctx.close_db();
}

#[test]
fn test_otp_rename() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create an OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("oldotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Rename the OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("mv").arg("oldotp").arg("newotp");

    cmd.assert().success();

    // Verify new name exists
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("newotp");
    cmd.assert().success();

    // Verify old name doesn't exist
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("oldotp");
    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_otp_copy() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create an OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("sourceotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Copy the OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("cp").arg("sourceotp").arg("destotp");

    cmd.assert().success();

    // Verify both exist
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("sourceotp");
    cmd.assert().success();

    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("destotp");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_otp_delete() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create an OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("deleteotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Delete the OTP entry
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("rm").arg("deleteotp");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "y").ok(); // Confirm deletion
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "Delete should succeed");

    // Verify it's gone
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("show").arg("deleteotp");
    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_otp_find() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create OTP entries
    for name in &["google_auth", "github_auth", "gitlab_auth"] {
        let mut cmd = ctx.cmd();
        cmd.arg("otp").arg("new").arg(name);
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
        }
        child.wait().ok();
    }

    // Search for items containing "git"
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("find").arg("git");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("github_auth"))
        .stdout(predicate::str::contains("gitlab_auth"));

    ctx.close_db();
}

#[test]
fn test_otp_categories() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create OTP entries in categories
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("new").arg("work/aws");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "JBSWY3DPEHPK3PXP").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // List work category
    let mut cmd = ctx.cmd();
    cmd.arg("otp").arg("ls").arg("work");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("aws"));

    ctx.close_db();
}
