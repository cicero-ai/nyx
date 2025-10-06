
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

pub struct TestContext {
    pub dbfile: String,
    pub port: u16,
    pub password: String,
}

impl TestContext {
    pub fn new() -> Self {
        let dbfile = "/tmp/nyx_test.db";
        let _ = std::fs::remove_file(&Path::new(&dbfile));

        Self {
            dbfile: dbfile.to_string(),
            port: 7924,
            password: "password123".to_string(),
        }
    }

    /// Get the nyx binary path
    fn nyx_bin() -> String {
        env!("CARGO_BIN_EXE_nyx").to_string()
    }

    /// Build a Command with common flags
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::new(Self::nyx_bin());
        cmd.arg("-f").arg(&self.dbfile);
        cmd.env("RUST_BACKTRACE", "1");
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd
    }

    /// Run a command and check for password errors
    pub fn _run_cmd(&self, cmd: &mut Command, context: &str) -> std::process::Output {
        let output = cmd.output().expect(&format!("Failed to execute command in {}", context));
        check_for_password_error(&output, context);
        output
    }

    /// Create the test database and start daemon
    pub fn create_db(&self) {
        let mut cmd = self.cmd();
        cmd.arg("test").arg("createdb").arg(&self.dbfile.to_string()).arg(self.password.to_string());
        cmd.stdin(Stdio::piped());

        // Send command
        let child = cmd.spawn().expect("Failed to spawn nyx");
        let output = child.wait_with_output().expect("Failed to wait for db create");

        if !output.status.success() {
            panic!("Failed to create database:\nstdout: {}\nstderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        }

        // Check if database was created
        if !Path::new(&self.dbfile).exists() {
            panic!("Database file was not created at {}!\nstdout: {}\nstderr: {}", self.dbfile, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        }

        // Verify daemon is running
        thread::sleep(Duration::from_millis(200));
        for attempt in 0..20 {
            let mut check_cmd = self.cmd();
            check_cmd.arg("db").arg("stats");
            let check_result = check_cmd.output();

            if let Ok(output) = &check_result {
                check_for_password_error(output, &format!("daemon verification attempt {}", attempt + 1));
                if output.status.success() {
                    return; // Daemon is up!
                }
            }

            thread::sleep(Duration::from_millis(200));
            if attempt == 19 {
                let last_check = check_cmd.output();
                let (stdout, stderr) = if let Ok(out) = last_check {
                    (String::from_utf8_lossy(&out.stdout).to_string(),
                     String::from_utf8_lossy(&out.stderr).to_string())
                } else {
                    ("N/A".to_string(), "N/A".to_string())
                };

                panic!("Daemon failed to start after database creation after {} attempts.\nDB exists: {}\nPort: {}\nLast stats check:\nstdout: {}\nstderr: {}",
                    attempt + 1,
                    self.dbfile, 
                    "",
                    stdout,
                    stderr);
            }
        }
    }

    /// Open an existing database
    #[allow(dead_code)]
    pub fn open_db(&self) {
        let mut cmd = self.cmd();
        cmd.arg("db").arg("open");
        cmd.stdin(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn nyx");

        // Provide password input
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "{}", self.password).ok();
        }

        let output = child.wait_with_output().expect("Failed to wait for nyx");
        check_for_password_error(&output, "db open");

        assert!(output.status.success(), "Failed to open database:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));

        // Give daemon time to start
        thread::sleep(Duration::from_millis(200));
    }

    /// Close the database and shutdown daemon
    pub fn close_db(&self) {
        let mut cmd = self.cmd();
        cmd.arg("db").arg("close");
        let _ = cmd.output();

        // Give daemon time to shutdown
        thread::sleep(Duration::from_millis(200));
    }

    /// Check if daemon is running
    #[allow(dead_code)]
    pub fn is_daemon_running(&self) -> bool {
        let mut cmd = self.cmd();
        cmd.arg("db").arg("stats");
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Ensure daemon is closed
        self.close_db();

        // Clean up database file
        let _ = std::fs::remove_file(&Path::new(&self.dbfile));
    }
}

/// Helper to wait for daemon to be ready
#[allow(dead_code)]
pub fn wait_for_daemon(ctx: &TestContext, max_attempts: u32) -> bool {
    for _ in 0..max_attempts {
        if ctx.is_daemon_running() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

/// Check if output contains invalid password error and panic if so
pub fn check_for_password_error(output: &std::process::Output, context: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stdout.contains("Invalid password, please double check and try again.")
        || stderr.contains("Invalid password, please double check and try again.") {
        panic!(
            "Test failed: Invalid password prompt detected in {}\nstdout: {}\nstderr: {}",
            context, stdout, stderr
        );
    }

    if stdout.contains("Password:") || stderr.contains("Password:") {
        panic!(
            "Test failed: Unexpected password prompt in {}\nstdout: {}\nstderr: {}",
            context, stdout, stderr
        );
    }
}
