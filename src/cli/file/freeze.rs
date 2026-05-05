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
pub struct CliFileFreeze {}

impl CliCommand for CliFileFreeze {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify any files to freeze.\n");
            cli_info!("    Usage: nyx freeze <FILE1> <FILE2> <MINUTES>\n");
            return Err(CliError::MissingParams.into());
        }

        let Some(fuse_dir) = crate::rpc::fs_launcher::get_mount_dir() else {
            return Err(CliError::Generic("Unable to determine fuse mount dir.".to_string()).into());
        };

        let Ok(minutes) = req.args.last().unwrap().parse::<u64>() else {
            cli_error!("The last argument must be the number of minutes to freeze files for.");
            return Err(CliError::MissingParams.into());
        };

        let mut hashes: Vec<String> = Vec::new();
        for (idx, filename) in req.args.iter().enumerate() {
            if (idx + 1) == req.args.len() {
                continue;
            }
            if filename.as_str() == "all" {
                hashes.push("all".to_string());
                continue;
            }

            if !Path::new(filename).exists() {
                return Err(CliError::InvalidParam(idx, format!("File does not exist, {}", filename)).into());
            }

            let full_path = if let Ok(pbuf) = fs::canonicalize(filename)
                && let Some(path_str) = pbuf.to_str()
            {
                path_str.to_string()
            } else {
                return Err(CliError::InvalidParam(idx, format!("File does not exist, {}", filename)).into());
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
        if let Err(e) = rpc::send::<&String, bool>("file.freeze", &vec![&json_str, &format!("{}", minutes)]) {
            return Err(Error::Generic(format!("Unable to freeze file: {}", e)).into());
        }

        cli_info!(
            "The following files are frozen and no longer protected for{} minutes:\n",
            minutes
        );
        for (idx, filename) in req.args.iter().enumerate() {
            if (idx + 1) == req.args.len() {
                continue;
            }
            cli_info!("    {}", filename);
        }

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Freeze File",
            "nyx freeze <FILE> <MINUTES>",
            "Freeze a protected file allowing any process to access it for a specified number of minutes.\n\nNOTE: You may specify 'all' as the filename which will freeze all protected files.",
        );

        help.add_param("FILE", "The file to freeze, relative or absolute");
        help.add_param("MINUTES", "Number of minutes to leave file unprotected.");
        help.add_example("nyx freeze config.yaml 5");
        help.add_example("nyx freeze all 5");
        help
    }
}
