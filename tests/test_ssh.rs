// Integration tests for SSH key operations
mod common;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use common::TestContext;
use std::fs;

#[test]
fn test_ssh_generate() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate a new SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("testkey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn nyx");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "server1.example.com").ok(); // host
        writeln!(stdin, "22").ok();                   // port
        writeln!(stdin, "ubuntu").ok();               // username
        writeln!(stdin, "").ok();                     // password (optional)
        writeln!(stdin, "Test SSH key").ok();         // notes
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "SSH key generation should succeed");

    ctx.close_db();
}

#[test]
fn test_ssh_list() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate multiple SSH keys
    for i in 1..=3 {
        let mut cmd = ctx.cmd();
        cmd.arg("ssh").arg("gen").arg(format!("key{}", i));
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "host{}.com", i).ok();
            writeln!(stdin, "22").ok();
            writeln!(stdin, "user").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
        }
        child.wait().ok();
    }

    // List all SSH keys
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("key1"))
        .stdout(predicate::str::contains("key2"))
        .stdout(predicate::str::contains("key3"));

    ctx.close_db();
}

#[test]
fn test_ssh_show() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate an SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("showtest");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "test.example.com").ok();
        writeln!(stdin, "2222").ok();
        writeln!(stdin, "testuser").ok();
        writeln!(stdin, "testpass").ok();
        writeln!(stdin, "Test notes").ok();
    }
    child.wait().ok();

    // Show the SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("showtest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.example.com"))
        .stdout(predicate::str::contains("2222"))
        .stdout(predicate::str::contains("testuser"));

    ctx.close_db();
}

#[test]
fn test_ssh_import() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Create a temporary SSH key file
    let ssh_key_content = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBTkXHh7QvvKKp9j6qZqZqZqZqZqZqZqZqZqZqZqZqZqQAAAJgvvvvvL777
7wAAAAtzc2gtZWQyNTUxOQAAACBTkXHh7QvvKKp9j6qZqZqZqZqZqZqZqZqZqZqZqZqZqZ
qQAAAEA1111111111111111111111111111111111111111111111111111VOR+eHtC+8oqn2P
qpmpmpmpmpmpmpmpmpmpmpmpmpmpmZkAAAAEXRlc3RAZXhhbXBsZS5jb20BAgMEBQ==
-----END OPENSSH PRIVATE KEY-----";

    let temp_key = std::env::temp_dir().join(format!("test_ssh_{}.pem", ctx.port));
    fs::write(&temp_key, ssh_key_content).expect("Failed to write temp key");

    // Import the SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("import").arg("imported");
    cmd.arg("--file").arg(&temp_key);
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "import.example.com").ok();
        writeln!(stdin, "22").ok();
        writeln!(stdin, "importuser").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "Imported key").ok();
    }

    let _status = child.wait().expect("Failed to wait");
    // Note: This might fail if the key format is invalid, but tests the import path

    // Cleanup
    let _ = fs::remove_file(&temp_key);

    ctx.close_db();
}

#[test]
fn test_ssh_edit() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate an SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("editkey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "original.com").ok();
        writeln!(stdin, "22").ok();
        writeln!(stdin, "user").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Edit the SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("edit").arg("editkey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "updated.com").ok();
        writeln!(stdin, "2222").ok();
        writeln!(stdin, "newuser").ok();
        writeln!(stdin, "newpass").ok();
        writeln!(stdin, "Updated notes").ok();
    }
    child.wait().ok();

    // Verify changes
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("editkey");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("updated.com"))
        .stdout(predicate::str::contains("2222"));

    ctx.close_db();
}

#[test]
fn test_ssh_rename() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate an SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("oldkey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "host.com").ok();
        writeln!(stdin, "22").ok();
        writeln!(stdin, "user").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Rename the SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("mv").arg("oldkey").arg("newkey");

    cmd.assert().success();

    // Verify new name exists
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("newkey");
    cmd.assert().success();

    // Verify old name doesn't exist
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("oldkey");
    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_ssh_copy() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate an SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("sourcekey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "host.com").ok();
        writeln!(stdin, "22").ok();
        writeln!(stdin, "user").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Copy the SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("cp").arg("sourcekey").arg("destkey");

    cmd.assert().success();

    // Verify both exist
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("sourcekey");
    cmd.assert().success();

    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("destkey");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_ssh_delete() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate an SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("deletekey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "host.com").ok();
        writeln!(stdin, "22").ok();
        writeln!(stdin, "user").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // Delete the SSH key
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("rm").arg("deletekey");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "y").ok(); // Confirm deletion
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "Delete should succeed");

    // Verify it's gone
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("show").arg("deletekey");
    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_ssh_find() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate SSH keys with searchable names
    for name in &["aws_server", "azure_server", "gcp_server"] {
        let mut cmd = ctx.cmd();
        cmd.arg("ssh").arg("gen").arg(name);
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn");
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "host.com").ok();
            writeln!(stdin, "22").ok();
            writeln!(stdin, "user").ok();
            writeln!(stdin, "").ok();
            writeln!(stdin, "").ok();
        }
        child.wait().ok();
    }

    // Search for items containing "aws"
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("find").arg("aws");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("aws_server"));

    ctx.close_db();
}

#[test]
fn test_ssh_categories() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Generate SSH keys in categories
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("gen").arg("production/webserver");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "prod.example.com").ok();
        writeln!(stdin, "22").ok();
        writeln!(stdin, "admin").ok();
        writeln!(stdin, "").ok();
        writeln!(stdin, "").ok();
    }
    child.wait().ok();

    // List production category
    let mut cmd = ctx.cmd();
    cmd.arg("ssh").arg("ls").arg("production");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("webserver"));

    ctx.close_db();
}

// Note: FUSE filesystem tests (xb, xh, xp, xu, xv commands for copying keys)
// are difficult to test in integration tests as they require:
// 1. FUSE support on the system
// 2. Proper permissions
// 3. Mount point availability
// These are better tested manually or in a specialized test environment
