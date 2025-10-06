// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::SshKey;
use crate::error::Error;
use crate::rpc;
use falcon_cli::*;
use ssh_key::PrivateKey;
use std::fs;

#[derive(Default)]
pub struct CliSshKeyImport {}

impl CliCommand for CliSshKeyImport {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name for the new entry.\n");
            cli_info!("    Usage: nyx ssh import <NAME> --file <PEM_FILE>\n");
            return Err(CliError::MissingParams.into());
        }
        req.validate_flag("--file", CliFormat::File)?;

        // Ensure item not exists
        cli::check_exists("ssh", &req.args[0], false)?;

        // GEt SSH key
        let filename = req.get_flag("--file").ok_or(CliError::MissingFlag("--file".to_string()))?;
        let private_key = fs::read(&filename).unwrap();

        // Parse SSH key, get public key
        let privkey =
            PrivateKey::from_openssh(&String::from_utf8(private_key.clone())?).map_err(|_| {
                CliError::Generic(
                    "Invalid private key, please double check and try again.".to_string(),
                )
            })?;
        let public_key = privkey.public_key().to_openssh().map_err(|e| {
            CliError::Generic(format!(
                "Unable to convert private SSH key to public: {}",
                e
            ))
        })?;

        // Get item info
        cli_header("Import SSH Key");
        cli_info!("Enter the new SSH key information below.  Leave blank to omit a field.\n");
        let host = cli_get_input("Host: ", "");
        let port = cli_get_input("Port [22]: ", "22");
        let username = cli_get_input("Username [root]: ", "root");
        let password = cli_get_password("Password (optional): ", true);
        let notes = cli_get_multiline_input("Notes");

        // Instantiate item
        let ssh_key = SshKey {
            display_name: req.args[0].to_string(),
            ino: 0,
            host,
            port: port.parse::<u16>()?,
            username,
            password,
            public_key,
            private_key,
            notes,
        };

        let key_str = serde_json::to_string(&ssh_key)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        if let Err(e) =
            rpc::send::<&String, bool>("ssh.import", &vec![&req.args[0].to_lowercase(), &key_str])
        {
            return Err(Error::Generic(format!("Unable to import SSH key: {}", e)).into());
        }

        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Import SSH Key",
            "nyx ssh import <NAME> -- file <PEM_FILE>",
            "Imports a new SSH key, which is then available as the IdentityFile paramter in your ~/.ssh/config file at: /tmp/nyx/ssh_keys/<NAME>, or the virtual directory you specified during database creation.",
        );

        help.add_param(
            "NAME",
            "Name of entry to add.  Supports directory structure (eg. category/myuser)",
        );
        help.add_example("nyx ssh import mysite/app_server --file /path/to/app_serevr.pem");
        help
    }
}
