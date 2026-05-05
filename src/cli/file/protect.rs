// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use sha2::{Sha256, Digest};
use crate::cli;
use crate::database::ProtectedFile;
use crate::error::Error;
use crate::rpc;
use falcon_cli::*;
use std::{fs, env};
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

#[derive(Default)]
pub struct CliFileProtect {}

impl CliCommand for CliFileProtect {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a filename to protect.\n");
            cli_info!("    Usage: nyx protect <FILE1> <FILE2> <FILE3>\n");
            return Err(CliError::MissingParams.into());
        }

        // Get files
        let mut files: Vec<ProtectedFile> = Vec::new();
        for (idx, filename) in req.args.iter().enumerate() {
            cli::check_exists("file", filename, false)?;
            if !Path::new(filename).exists() {
                return Err(CliError::InvalidParam(idx, format!("File does not exist, {}", filename)).into());
            }

            let full_path = if let Ok(pbuf) = fs::canonicalize(filename) && let Some(path_str) = pbuf.to_str() {
                path_str.to_string()
            } else {
                return Err(CliError::InvalidParam(idx, format!("File does not exist, {}", filename)).into())
            };

            let mut hasher = Sha256::new();
            hasher.update(filename);
            let file_hash = hasher.finalize();

            files.push(ProtectedFile {
                filename: full_path,
                file_hash: format!("{:x}", file_hash),
                ino: 0,
                is_protected: true,
                contents: vec![],
                whitelist: vec![]
            });
        }

        // Get item info
        cli_header("Protect File");
        cli_sendln!("Enter the the binaries that are allowed to access this file, one binary per-line (ie. gh, aws, claude).");   
        let whitelist_vec = cli_get_multiline_input("Allowed Binaries");

        let mut whitelist: Vec<String> = Vec::new();
        for app in whitelist_vec.split("\n") {
            let Some(binary) = self.which(app) else {
                return Err(CliError::Generic(format!("Unable to find executable path of {}", app)).into())
            };
            whitelist.push(binary);
        }

        for file in files.iter_mut() {
            file.whitelist = whitelist.clone();
        }

        let file_str = serde_json::to_string(&files)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        if let Err(e) =
            rpc::send::<&String, bool>("file.new", &vec![&file_str])
        {
            return Err(Error::Generic(format!("Unable to protect file: {}", e)).into());
        }

        cli_info!("Protected  the files:\n  ");
        for file in files.iter() {
            cli_info!("    {}", file.filename);
        }

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Protect File",
            "nyx protect <FILE1> <FILE2> <FILE3>...",
            "Puts specified files under protection ensuring only whitelisted binaries can access them.",
        );

        help.add_param("FILE", "File to protect, relative or absolute");
        help.add_example("nyx protect config.yaml");

        help
    }

}


impl CliFileProtect {

    fn which(&self, binary: &str) -> Option<String> {

        if binary.contains('/') {
            let path = PathBuf::from(binary);
            return if self.is_executable(&path) && let Some(res) = path.to_str() { Some(res.to_string()) } else { None };
        }

        let path_var = env::var_os("PATH")?;
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(binary);
            if self.is_executable(&candidate) {
                let pbuf = match candidate.canonicalize() {
                    Ok(r) => r,
                    Err(_) => candidate
                };
                return Some(pbuf.to_str()?.to_string());
            }
        }

        None
    }

    fn is_executable(&self, path: &Path) -> bool {
        fs::metadata(path)
            .map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
            .unwrap_or(false)
    }

}


