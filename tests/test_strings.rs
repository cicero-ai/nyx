// Integration tests for string storage operations
mod common;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use common::TestContext;

#[test]
fn test_string_set() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set a string value
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("api_key").arg("sk_test_12345");

    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_string_get() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set a string value
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("test_key").arg("test_value_123");
    cmd.assert().success();

    // Get the string value
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("test_key");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_value_123"));

    ctx.close_db();
}

#[test]
fn test_string_list() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set multiple strings
    for i in 1..=3 {
        let mut cmd = ctx.cmd();
        cmd.arg("set").arg(format!("key{}", i)).arg(format!("value{}", i));
        cmd.assert().success();
    }

    // List all strings
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("key1"))
        .stdout(predicate::str::contains("key2"))
        .stdout(predicate::str::contains("key3"));

    ctx.close_db();
}

#[test]
fn test_string_update() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set initial value
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("update_test").arg("original_value");
    cmd.assert().success();

    // Update with new value
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("update_test").arg("updated_value");
    cmd.assert().success();

    // Verify new value
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("update_test");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("updated_value"));

    ctx.close_db();
}

#[test]
fn test_string_rename() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set a string
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("old_name").arg("test_value");
    cmd.assert().success();

    // Rename it
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("mv").arg("old_name").arg("new_name");
    cmd.assert().success();

    // Verify old name doesn't exist
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("old_name");
    cmd.assert().failure();

    // Verify new name exists
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("new_name");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_string_copy() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set a string
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("source").arg("copy_value");
    cmd.assert().success();

    // Copy it
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("cp").arg("source").arg("destination");
    cmd.assert().success();

    // Verify both exist
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("source");
    cmd.assert().success();

    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("destination");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_string_delete() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set a string
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("to_delete").arg("delete_me");
    cmd.assert().success();

    // Delete it
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("rm").arg("to_delete");
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "y").ok(); // Confirm deletion
    }

    let status = child.wait().expect("Failed to wait");
    assert!(status.success(), "Delete should succeed");

    // Verify it's gone
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("to_delete");
    cmd.assert().failure();

    ctx.close_db();
}

#[test]
fn test_string_find() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set multiple strings with searchable names
    for name in &["stripe_api_key", "stripe_secret", "paypal_api_key"] {
        let mut cmd = ctx.cmd();
        cmd.arg("set").arg(name).arg("value");
        cmd.assert().success();
    }

    // Search for items containing "stripe"
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("find").arg("stripe");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stripe_api_key"))
        .stdout(predicate::str::contains("stripe_secret"));

    ctx.close_db();
}

#[test]
fn test_string_categories() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set strings in categories
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("aws/access_key").arg("AKIA123");
    cmd.assert().success();

    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("aws/secret_key").arg("secret123");
    cmd.assert().success();

    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("gcp/api_key").arg("gcp123");
    cmd.assert().success();

    // List root - should show categories
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("aws/"))
        .stdout(predicate::str::contains("gcp/"));

    // List aws category
    let mut cmd = ctx.cmd();
    cmd.arg("str").arg("ls").arg("aws");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("access_key"))
        .stdout(predicate::str::contains("secret_key"));

    ctx.close_db();
}

#[test]
fn test_string_special_characters() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set string with special characters
    let special_value = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("special").arg(special_value);
    cmd.assert().success();

    // Get it back
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("special");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(special_value));

    ctx.close_db();
}

#[test]
fn test_string_empty_value() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set empty string
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("empty_key").arg("");
    cmd.assert().success();

    // Get it back
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("empty_key");
    cmd.assert().success();

    ctx.close_db();
}

#[test]
fn test_string_long_value() {
    let ctx = TestContext::new();
    ctx.create_db();

    // Set a long string value
    let long_value = "a".repeat(1000);
    let mut cmd = ctx.cmd();
    cmd.arg("set").arg("long_key").arg(&long_value);
    cmd.assert().success();

    // Get it back
    let mut cmd = ctx.cmd();
    cmd.arg("get").arg("long_key");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&long_value));

    ctx.close_db();
}
