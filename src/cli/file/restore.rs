// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::error::Error;
use crate::rpc;
use falcon_cli::*;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct CliFileRestore {}

impl CliCommand for CliFileRestore {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name for the new entry.\n");
            cli_info!("    Usage: nyx restore <FILE1> <FILE2>\n");
            return Err(CliError::MissingParams.into());
        }

        let Some(fuse_dir) = crate::rpc::fs_launcher::get_mount_dir() else {
            return Err(CliError::Generic("Unable to determine fuse mount dir.".to_string()).into())
        };

        let mut hashes: Vec<String> = Vec::new();
        for (idx, filename) in req.args.iter().enumerate() {
            if filename.as_str() == "all" {
                hashes.push("all".to_string());
                continue;
            } else if !Path::new(filename).exists() {
                return Err(CliError::InvalidParam(idx, format!("File does not exist, {}", filename)).into());
            }

            let full_path = if let Ok(pbuf) = fs::canonicalize(filename) && let Some(path_str) = pbuf.to_str() {
                path_str.to_string()
            } else {
                return Err(CliError::InvalidParam(idx, format!("File does not exist, {}", filename)).into())
            };

            if !full_path.starts_with(&fuse_dir) {
                return Err(CliError::Generic(format!("File is not a protected file, {}", filename)).into());
            }

            let file_hash = full_path.trim_start_matches(&format!("{}/", fuse_dir)).to_string();
            hashes.push(file_hash);
        }

        let json_str = serde_json::to_string(&hashes)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        if let Err(e) =
            rpc::send::<&String, bool>("file.delete", &vec![&json_str])
        {
            return Err(Error::Generic(format!("Unable to restore file: {}", e)).into());
        }

        cli_info!("The following files are restored and no longer protected:\n");
        for filename in req.args.iter() {
            cli_info!("    {}", filename);
        }

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Restore File",
            "nyx restore <FILE>",
            "Restore a previously protected file to its original unprotected state.\n\nNOTE: You may specify 'all' as the filename which will restore all protected files."
        );

        help.add_param("FILE", "The file to restore, relative or absolute");
        help.add_example("nyx restore config.yaml");
        help.add_example("nyx restore all");
        help
    }
}
