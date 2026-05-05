// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use falcon_cli::*;
use std::io::Write;
use std::process::{Command, Stdio};

/// Copy text to clipboard
pub fn copy(text: &str) -> Result<(), Error> {
    // Get available tools to try
    let mut _tools: Vec<(&str, Vec<&str>)> = vec![];

    #[cfg(target_os = "linux")]
    {
        _tools = vec![
            ("xclip", vec!["-selection", "clipboard", "-i"]),
            ("xsel", vec!["--clipboard", "--input"]),
            ("wl-copy", vec![]),
        ];
    }

    #[cfg(target_os = "macos")]
    {
        _tools.push(("pbcopy", vec![]));
    }

    #[cfg(target_os = "windows")]
    {
        _tools.push(("clip", vec![]));
    }

    // Iterate through tools
    for (cmd, args) in &_tools {
        if let Ok(mut child) =
            Command::new(cmd).args(args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
            && let Some(mut stdin) = child.stdin.take()
            && stdin.write_all(text.as_bytes()).is_ok()
        {
            drop(stdin);
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                cli_sendln!("Copied to clipboard");
                return Ok(());
            }
        }
    }

    // Failed
    cli_warn!("Supported clipboard not found, outputting to terminal.");
    cli_warn!("To resolve, install xclip:  sudo apt -y install xclip\n");
    cli_sendln!("{}", text);

    Ok(())
}

/// Clear all clipboard selections (CLIPBOARD, PRIMARY, and Wayland).
/// Called on clipboard timeout to remove secrets from clipboard.
pub fn clear() {
    #[cfg(target_os = "linux")]
    {
        // Clear X11 CLIPBOARD selection
        pipe_to_cmd("xclip", &["-selection", "clipboard", "-i"], b"");
        // Clear X11 PRIMARY selection (middle-click paste)
        pipe_to_cmd("xclip", &["-selection", "primary", "-i"], b"");
        // xsel fallback
        pipe_to_cmd("xsel", &["--clipboard", "--clear"], b"");
        pipe_to_cmd("xsel", &["--primary", "--clear"], b"");
        // Wayland: --clear relinquishes ownership properly (no empty-string hack)
        let _ = Command::new("wl-copy").arg("--clear").stdout(Stdio::null()).stderr(Stdio::null()).status();
    }

    #[cfg(target_os = "macos")]
    {
        // pbcopy with empty input clears the pasteboard
        pipe_to_cmd("pbcopy", &[], b"");
    }

    #[cfg(target_os = "windows")]
    {
        pipe_to_cmd("clip", &[], b"");
    }
}

/// Pipe bytes into a command's stdin. Silently ignores failures (tool may not be installed).
fn pipe_to_cmd(cmd: &str, args: &[&str], data: &[u8]) {
    if let Ok(mut child) =
        Command::new(cmd).args(args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
        && let Some(mut stdin) = child.stdin.take()
    {
        let _ = stdin.write_all(data);
        drop(stdin);
        let _ = child.wait();
    }
}
