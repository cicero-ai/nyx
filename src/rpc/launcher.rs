// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::RpcDaemon;
use crate::database::NyxDb;
use crate::{CONFIG, Error};
use base64::{Engine as _, engine::general_purpose};
use falcon_cli::*;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::net::{SocketAddr, TcpStream};
#[cfg(not(target_os = "windows"))]
use std::process::Stdio;
use std::process::{Command, exit};
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};
use tokio::runtime::Runtime;
use zeroize::Zeroize;

#[cfg(unix)]
use nix::libc;
#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(windows)]
use winapi::um::handleapi::CloseHandle;
#[cfg(windows)]
use winapi::um::processthreadsapi::CreateProcessW;
#[cfg(windows)]
use winapi::um::processthreadsapi::PROCESS_INFORMATION;
#[cfg(windows)]
use winapi::um::processthreadsapi::STARTUPINFOW;
#[cfg(windows)]
use winapi::um::winbase::CREATE_NEW_PROCESS_GROUP;
#[cfg(windows)]
use winapi::um::winbase::DETACHED_PROCESS;

/// Launch the RPC daemon
/// Run start command, launch RPC daemon
pub fn launch(dbfile: &str, n_password: [u8; 32]) -> Result<(), Error> {
    // Ping
    if ping() {
        let _ = super::send::<String, bool>("db.close", &vec![]);
        thread::sleep(Duration::from_millis(300));
    }

    // Base64
    let hashed_password = general_purpose::STANDARD.encode(n_password);

    #[cfg(any(target_os = "linux", feature = "fuse"))]
    // Check for unmount
    {
        if super::fs_launcher::is_mount_point() {
            super::fs_launcher::unmount()?;
        }
    }

    // Open log file
    let log_filename = get_logfile_path();
    let log_file = OpenOptions::new().create(true).append(true).open(&log_filename)?;
    let err_file = log_file.try_clone()?;

    // Get arguments
    let mut cmd_args = vec![];
    let mut include_next = false;
    for value in env::args() {
        if include_next {
            cmd_args.push(value.to_string());
            include_next = false;
        } else if [
            "-f",
            "--dbfile",
            "-h",
            "--host",
            "-p",
            "--port",
            "-t",
            "--timeout",
            "-c",
            "--cb-timeout",
            "-m",
            "--mount-dir",
        ]
        .contains(&value.as_str())
        {
            cmd_args.push(value.to_string());
            include_next = true;
        }
    }
    cmd_args.push("-d".to_string());

    // Define command to spawn child
    let run_cmd = env::args().next().unwrap();
    let mut cmd = Command::new(&run_cmd);
    cmd.args(cmd_args);

    // Set environment vars for child command only
    cmd.env("NYX_LAUNCH_HASH", &hashed_password);
    cmd.env("NYX_LAUNCH_DBFILE", dbfile);

    // Set up the command to detach
    #[cfg(unix)]
    {
        let mut child = cmd.stdin(Stdio::null()).stdout(log_file).stderr(err_file).spawn()?;

        match child.try_wait() {
            Ok(None) => unsafe {
                libc::setsid();
            },
            Ok(Some(status)) => {
                let output = child.wait_with_output().expect("Failed to wait for child output");
                let errmsg = format!(
                    "Unexpected error when starting RPC daemon, status {}, stdout {}, stderr {}",
                    status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(Error::Rpc(errmsg));
            }
            Err(e) => {
                return Err(Error::Rpc(format!(
                    "Unable to detach daemon child process with pid {}, error: {}",
                    child.id(),
                    e
                )));
            }
        };
    }

    #[cfg(windows)]
    {
        let command = OsString::from("your_command");
        let mut command_wide: Vec<u16> = command.encode_wide().collect();
        command_wide.push(0);

        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

        let success = unsafe {
            CreateProcessW(
                ptr::null(), // No module name (use command line)
                command_wide.as_mut_ptr(),
                ptr::null_mut(),                             // Process handle not inheritable
                ptr::null_mut(),                             // Thread handle not inheritable
                0,                                           // Set handle inheritance to FALSE
                DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP, // Detach and create new process group
                ptr::null_mut(),                             // Use parent's environment block
                ptr::null(),                                 // Use parent's starting directory
                &mut startup_info,
                &mut process_info,
            )
        };

        if success == 0 {
            let error_code = unsafe { GetLastError() };
            cli_error!("Failed to start the daemon, error code: {}", error_code);
            std::process::exit(1);
        } else {
            unsafe {
                CloseHandle(process_info.hProcess);
                CloseHandle(process_info.hThread);
            }
        }
    }

    // Wait for daemon to start
    let mut started = false;
    for _ in 0..25 {
        thread::sleep(Duration::from_millis(200));
        if ping() {
            started = true;
            break;
        }
    }

    if !started {
        cli_error!(
            "Unable to start Nyx daemon due to unexpected error, please check {} for details.",
            log_filename
        );
        exit(1);
    }

    // Remove env vars
    unsafe {
        env::remove_var("NYX_LAUNCH_HASH");
        env::remove_var("NYX_LAUNCH_DBFILE");
    }

    // Checj fuse point
    #[cfg(any(target_os = "linux", feature = "fuse"))]
    {
        //super::fs_launcher::check_mount_successful();
    }

    Ok(())
}

/// Get logfile path
fn get_logfile_path() -> String {
    if let Some(dir) = dirs::data_dir()
        && let Some(datadir) = dir.to_str()
    {
        return format!("{}/nyx.log", datadir);
    }
    "nyx.log".to_string()
}

/// Ping, see if RPC daemon is online
pub fn ping() -> bool {
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port);
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1000)).is_ok()
}

/// Start daemon
pub fn start_daemon() -> Result<(), Error> {
    // Get environment variables
    let mut hashed_password =
        env::var("NYX_LAUNCH_HASH").map_err(|e| Error::Generic(format!("Environment variable error: {}", e)))?;
    let dbfile =
        env::var("NYX_LAUNCH_DBFILE").map_err(|e| Error::Generic(format!("Environment variable error: {}", e)))?;

    // Harden process security
    if let Err(e) = harden() {
        cli_warn!("Process hardening warning: {}", e);
    }

    // Decode base64
    let mut tmp_password = general_purpose::STANDARD
        .decode(&hashed_password)
        .map_err(|e| Error::Generic(format!("Base64 decode error: {}", e)))?;
    let mut n_password: [u8; 32] = tmp_password.clone().try_into().unwrap();

    // Zero out base64 intermediates immediately
    hashed_password.zeroize();
    tmp_password.zeroize();

    // Load database
    let db = NyxDb::load(&dbfile, n_password)?;

    // Start runtime
    let rt = Runtime::new()?;

    // Start daemon -- n_password is copied into RpcSession.lock (Zeroizing wrapper)
    rt.block_on(async {
        let daemon = Arc::new(RpcDaemon::new(db, &dbfile, n_password));
        n_password.zeroize();
        if let Err(e) = daemon.start().await {
            cli_error!("Unable to start RPC daemon: {}", e);
        }
    });

    Ok(())
}

/// Harden process: umask, raise RLIMIT_MEMLOCK, disable core dumps / ptrace
fn harden() -> Result<(), Error> {
    #[cfg(unix)]
    {
        // Set umask: new files default to owner-only (0600)
        unsafe {
            libc::umask(0o177);
        }

        // Raise mlock limit to 1MB so SecureBuffer can lock secret pages in RAM
        let limit = libc::rlimit {
            rlim_cur: 1024 * 1024,
            rlim_max: 1024 * 1024,
        };
        let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &limit) };
        if ret != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    // Disable core dumps and block ptrace attachment
    #[cfg(target_os = "linux")]
    {
        let ret = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };
        if ret != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let ret = unsafe { libc::ptrace(libc::PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0) };
        if ret != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(())
}
