// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use sha2::{Sha256, Digest};
use crate::error::Error;
use crate::rpc;
use crate::database::{file_credentials, ProtectedFile};
use falcon_cli::*;
use std::{fs, env};
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

#[derive(Default)]
pub struct CliFileScan { }

impl CliCommand for CliFileScan {

    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        let mut found: Vec<(String, Vec<&'static str>)> = vec![];
        let search = if req.args.is_empty() { "".to_string() } else { req.args[0].to_string() };
        let candidates = file_credentials::get();

        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        for (name, apps) in candidates.iter() {
            if name.contains('*') {
                // Wildcard path — split into directory and pattern parts
                let full_pattern = config_dir.join(name);
                let parent = match full_pattern.parent() {
                    Some(p) => p.to_path_buf(),
                    None => continue,
                };
                let file_pattern = match full_pattern.file_name() {
                    Some(f) => f.to_string_lossy().to_string(),
                    None => continue,
                };

                if parent.is_dir()
                    && let Ok(entries) = std::fs::read_dir(&parent) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if let Some(file_name) = path.file_name() {
                                let file_name_str = file_name.to_string_lossy();
                                if matches_wildcard(&file_pattern, &file_name_str)
                                    && path.exists() {
                                        found.push((
                                            path.to_string_lossy().to_string(),
                                            apps.clone(),
                                        ));
                                    }
                            }
                        }
                    }
            } else {
                // Plain path, no wildcard
                let full_path = config_dir.join(name);
                if full_path.exists() {
                    found.push((
                        full_path.to_string_lossy().to_string(),
                        apps.clone(),
                    ));
                }
            }
        }

        if found.is_empty() {
            cli_info!("No credential files found.");
            return Ok(());
        }

        let (mut num, mut res, mut queue) = (1, vec![], vec![]);
        found.sort_by(|a,b| a.0.cmp(&b.0));
        for (path, apps) in found.iter() {
            let filename = path.replace(config_dir.to_str().unwrap(), "").trim_start_matches('/').to_string();
            if (!search.is_empty()) && !filename.starts_with(&search) { continue; }

            res.push(vec![format!("{}", num), filename, apps.join(", ").to_string()]);
            queue.push((path.to_string(), apps.clone()));
            num += 1;
        }

        cli_header("Scan Results");
        cli_sendln!("The following unprotected sensitive files  and their associated binaries have been found:\n");
        cli_display_table(&["#", "File", "Binaries"], &res);

        if !cli_confirm("Would you like to protect these files so only their associated binaries may access them?") {
            cli_sendln!("Ok, goodbye");
            return Ok(());
        }

        let mut files = vec![];
        for (full_path, apps) in queue {

            let whitelist = apps.iter().filter_map(|app| self.which(app)).collect::<Vec<String>>();

            let mut hasher = Sha256::new();
            hasher.update(&full_path);
            let file_hash = hasher.finalize();

            files.push(ProtectedFile {
                filename: full_path,
                file_hash: format!("{:x}", file_hash),
                ino: 0,
                is_protected: true,
                contents: vec![],
                whitelist
            });
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
            "Scan Potential Credential Files",
            "nyx scan <PARENT_DIR>",
            "Scans your local folders for known credential files that should be protected."
        );

        help.add_param("PARENT_DIR", "Optional parent dir relative to home directory to serach in.");
        help.add_example("nyx scan");
        help
    }
}

// Simple wildcard matcher supporting only * (matches any sequence of chars)
fn matches_wildcard(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }

    let mut remaining = name;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            // Pattern must start with this prefix
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
        } else if i == parts.len() - 1 {
            // Pattern must end with this suffix
            return remaining.ends_with(part);
        } else {
            // Find this part somewhere in the remaining string
            match remaining.find(part) {
                Some(pos) => remaining = &remaining[pos + part.len()..],
                None => return false,
            }
        }
    }

    true
}


impl CliFileScan {

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


