// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::RpcDaemon;
use crate::database::NyxFs;
use crate::{CONFIG, Error};
use falcon_cli::*;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Mount to fuse directory
pub fn mount(daemon: &RpcDaemon) -> Result<(), Error> {
    if !Path::new(&CONFIG.fuse_mount_dir).exists() {
        fs::create_dir_all(&CONFIG.fuse_mount_dir)?;
    }

    // Get options
    let options = if is_auto_unmount_enabled() {
        vec![fuser::MountOption::AutoUnmount]
    } else {
        vec![]
    };

    // Mount
    let fs_instance = NyxFs(Arc::clone(&daemon.nyxdb));
    let fuse_session = fuser::spawn_mount2(fs_instance, &CONFIG.fuse_mount_dir, &options)
        .map_err(|e| Error::Db(format!("Unable to mount fuse filesystem: {}", e)))?;
    *daemon.fuse_point.lock().unwrap() = Some(fuse_session);

    Ok(())
}

/// Get mount options
fn is_auto_unmount_enabled() -> bool {
    #[cfg(target_os = "linux")]
    {
        if !Path::new("/etc/fuse.conf").exists() {
            return false;
        }
        let contents = fs::read_to_string("/etc/fuse.conf").unwrap_or_default();
        contents.lines().any(|line| line.trim() == "user_allow_other")
    }

    #[cfg(target_os = "macos")]
    {
        return true;
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

/// Check whether or not directory is an orphaned mount point
pub fn is_mount_point(path: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        // Check /proc/mounts
        if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
            return mounts.lines().any(|line| line.split_whitespace().nth(1) == Some(path));
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Check mount output on macOS
        if let Ok(output) = std::process::Command::new("mount").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                return stdout.lines().any(|line| line.contains(path));
            }
        }
    }

    false
}

/// Unmoun orphaned mount point
pub fn unmount() -> Result<(), Error> {
    // Try regular unmoun first
    let unmount_result =
        std::process::Command::new("fusermount").arg("-u").arg(&CONFIG.fuse_mount_dir).output();

    if let Ok(output) = unmount_result
        && output.status.success()
    {
        return Ok(());
    }

    // Use sudo
    cli_send!(
        "An orphaned mount point from a previous session has been detected, and needs to be unmounted.\n"
    );
    let sudo_result =
        std::process::Command::new("sudo").arg("umount").arg(&CONFIG.fuse_mount_dir).status();

    if let Ok(status) = sudo_result
        && !status.success()
    {
        return Err(Error::Db(format!(
            "Failed to unmount {}. Please run: sudo umount {}",
            CONFIG.fuse_mount_dir, CONFIG.fuse_mount_dir
        )));
    }

    // Give system a moment to clean up
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(())
}

/// Check whether or not mount was successful
pub fn check_mount_successful() {
    let ssh_dir = format!("{}/ssh_keys", CONFIG.fuse_mount_dir);
    if Path::new(&ssh_dir).exists() {
        return;
    }
    cli_warn!("Unable to mount FUSE point.");
    #[cfg(target_os = "linux")]
    {
        if !Path::new("/etc/fuse.conf").exists() {
            cli_warn!("FUSE Filesystem is not installed, run the following command to resolve:\n");
            cli_warn!("    sudo apt -y install fuse fuse3\n");
        } else {
            cli_warn!("FUSE filesystem appears to be installed, unknown error.\n");
        }
    }
    #[cfg(target_os = "macos")]
    {
        let macfuse_installed = Path::new("/Library/Filesystems/macfuse.fs").exists() 
            || Path::new("/Library/Filesystems/osxfuse.fs").exists();
        
        if !macfuse_installed {
            cli_warn!("MacFUSE is not installed, to resolve visit https://macfuse.github.io/ for installation instructions.\n");
        } else {
            cli_warn!("MacFUSE appears to be installed, unknown error.\n");
        }
    }
}

