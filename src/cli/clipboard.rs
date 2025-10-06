// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use falcon_cli::*;
use std::process::{Command, Stdio};
use crate::Error;

/// Copy to clipboard
pub fn copy(text: &str) -> Result<(), Error> {

    // Get available tools to try
    let mut _tools: Vec<(&str, Vec<&str>)> = vec![];

    #[cfg(target_os = "linux")]
    {
            _tools = vec![
            ("xclip", vec!["-selection", "clipboard", "-i"]),
            ("xsel", vec!["--clipboard", "--input"]),
            ("wl-copy", vec![])
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
        if let Ok(mut child) = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                if stdin.write_all(text.as_bytes()).is_ok() {
                    drop(stdin);
                    if child.wait().map(|s| s.success()).unwrap_or(false) {
                        cli_sendln!("Copied to clipboard");
                        return Ok(());
                    }
                }
            }
        }
    }

    // Failed
    cli_warn!("Supported clipboard not found, outputting to terminal.");
    cli_warn!("To resolve, install xclip:  sudo apt -y install xclip\n");
    cli_sendln!("{}", text);

    Ok(())
}

