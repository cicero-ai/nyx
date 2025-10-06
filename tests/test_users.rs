// Integration tests for user/password management
mod common;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use common::TestContext;

#[test]
fn test_user_new() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a new user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("testuser");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn nyx");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "test_username").ok();
        writeln!(stdin, "test_password").ok();
        writeln!(stdin, "https://test.com").ok();
        writeln!(stdin, "test notes\n").ok();
        writeln!(stdin, "\n").ok();
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "User creation should succeed");

    ctx.close_db();
}

#[test]
fn test_user_list() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create multiple users
    for i in 1..=3 {
        let mut cmd = ctx.cmd();
        cmd.arg("new").arg(format!("user{}", i));
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "username{}", i).ok();
            writeln!(stdin, "password{}", i).ok();
            writeln!(stdin, "\n").ok();
            writeln!(stdin, "\n").ok();
        }
        child.wait().ok();
    }

    // List all users
    let mut cmd = ctx.cmd();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("user1"))
        .stdout(predicate::str::contains("user2"))
        .stdout(predicate::str::contains("user3"));

    ctx.close_db();
}

#[test]
fn test_user_show() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("showtest");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "myusername").ok();
        writeln!(stdin, "mypassword").ok();
        writeln!(stdin, "https://example.com").ok();
        writeln!(stdin, "Some notes").ok();
    }
    child.wait().ok();

    // Show the user
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("showtest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("myusername"))
        .stdout(predicate::str::contains("https://example.com"));

    ctx.close_db();
}

#[test]
fn test_user_edit() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("edituser");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "original_username").ok();
        writeln!(stdin, "original_password").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Edit the user
    let mut cmd = ctx.cmd();
    cmd.arg("edit").arg("edituser");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "updated_username").ok();
        writeln!(stdin, "updated_password").ok();
        writeln!(stdin, "https://updated.com").ok();
        writeln!(stdin, "Updated notes").ok();
    }
    child.wait().ok();

    // Verify changes
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("edituser");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("updated_username"));

    ctx.close_db();
}

#[test]
fn test_user_rename() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("oldname");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "username").ok();
        writeln!(stdin, "password").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Rename the user
    let mut cmd = ctx.cmd();
    cmd.arg("mv").arg("oldname").arg("newname");

    cmd.assert().success();

    // Verify old name doesn't exist
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("oldname");
    cmd.assert().failure();

    // Verify new name exists
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("newname");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_user_copy() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("source");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "username").ok();
        writeln!(stdin, "password").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Copy the user
    let mut cmd = ctx.cmd();
    cmd.arg("cp").arg("source").arg("destination");

    cmd.assert().success();

    // Verify both exist
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("source");
    cmd.assert().success();

    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("destination");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_user_delete() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a user
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("todelete");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "username").ok();
        writeln!(stdin, "password").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Delete the user
    let mut cmd = ctx.cmd();
    cmd.arg("rm").arg("todelete");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "y").ok(); // Confirm deletion
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "Delete should succeed");

    // Verify it's gone
    let mut cmd = ctx.cmd();
    cmd.arg("show").arg("todelete");
    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_user_find() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create users with searchable names
    for name in &["github_account", "gitlab_account", "bitbucket_account"] {
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

    // Search for items containing "git"
    let mut cmd = ctx.cmd();
    cmd.arg("find").arg("git");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("github_account"))
        .stdout(predicate::str::contains("gitlab_account"));

    ctx.close_db();
}

#[test]
fn test_user_categories() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create users in different categories
    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("work/gitlab");
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

    let mut cmd = ctx.cmd();
    cmd.arg("new").arg("personal/email");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "user").ok();
        writeln!(stdin, "pass").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // List root directory - should show categories
    let mut cmd = ctx.cmd();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("work/"))
        .stdout(predicate::str::contains("personal/"));

    // List work category
    let mut cmd = ctx.cmd();
    cmd.arg("ls").arg("work");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("gitlab"));

    ctx.close_db();
}
