// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::database::ProtectedFile;
use crate::rpc;
use falcon_cli::*;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Default)]
pub struct CliFileEdit {}

impl CliCommand for CliFileEdit {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a file to edit.\n");
            cli_info!("    Usage: nyx file edit <FILE>\n");
            return Err(CliError::MissingParams.into());
        }

        let Some(fuse_dir) = crate::rpc::fs_launcher::get_mount_dir() else {
            return Err(CliError::Generic("Unable to determine fuse mount dir.".to_string()).into());
        };

        if !Path::new(&req.args[0]).exists() {
            return Err(CliError::InvalidParam(0, format!("File does not exist, {}", req.args[0])).into());
        }

        let full_path = if let Ok(pbuf) = fs::canonicalize(&req.args[0])
            && let Some(path_str) = pbuf.to_str()
        {
            path_str.to_string()
        } else {
            return Err(CliError::InvalidParam(0, format!("File does not exist, {}", req.args[0])).into());
        };

        if !full_path.starts_with(&fuse_dir) {
            return Err(CliError::Generic(format!("File is not a protected file, {}", req.args[0])).into());
        }
        let file_hash = full_path.trim_start_matches(&format!("{}/", fuse_dir)).to_string();

        // Get file
        let mut file: ProtectedFile = rpc::send("file.get", &vec![&file_hash])?;

        // Get file info
        cli_header(&format!("Edit File {}", req.args[0]));
        cli_info!("The file is currently allowed  to be accessed by the following binaries:\n");
        for app in file.whitelist.iter() {
            cli_info!("    {}", app);
        }
        println!();

        cli_sendln!(
            "Enter the the binaries that are allowed to access this file, one binary per-line (ie. gh, aws, claude)."
        );
        let whitelist_vec = cli_get_multiline_input("Allowed Binaries");

        file.whitelist.clear();
        for app in whitelist_vec.split("\n") {
            let Some(binary) = self.which(app) else {
                return Err(CliError::Generic(format!("Unable to find executable path of {}", app)).into());
            };
            file.whitelist.push(binary);
        }

        let json_str = serde_json::to_string(&file)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Edit file
        rpc::send::<&String, bool>("file.edit", &vec![&file.filename, &json_str])?;

        cli_info!("Updated allowed binaries for the file {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Edit Protected File",
            "nyx file edit <FILE>",
            "Edit file's allowed binaries.",
        );

        help.add_param("FILE", "Protected file to edit, relative or absolute path.");
        help.add_example("nyx file edit config.yaml");
        help
    }
}

impl CliFileEdit {
    fn which(&self, binary: &str) -> Option<String> {
        if binary.contains('/') {
            let path = PathBuf::from(binary);
            return if self.is_executable(&path)
                && let Some(res) = path.to_str()
            {
                Some(res.to_string())
            } else {
                None
            };
        }

        let path_var = env::var_os("PATH")?;
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(binary);
            if self.is_executable(&candidate) {
                let pbuf = match candidate.canonicalize() {
                    Ok(r) => r,
                    Err(_) => candidate,
                };
                return Some(pbuf.to_str()?.to_string());
            }
        }

        None
    }

    fn is_executable(&self, path: &Path) -> bool {
        fs::metadata(path).map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0)).unwrap_or(false)
    }
}
