// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::SshKey;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliSshKeyEdit {}

impl CliCommand for CliSshKeyEdit {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name to edit.\n");
            cli_info!("    Usage: nyx ssh edit <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure item exists
        cli::check_exists("ssh", &req.args[0], true)?;

        // Get item
        let mut ssh_key: SshKey = rpc::send("ssh.get", &vec![&req.args[0]])?;

        // Get item info
        cli_header(&format!("Edit {}", req.args[0]));
        cli_info!("Enter the new SSH key information below.  Leave blank to skip a field.\n");

        let host = cli_get_input("Host: ", "");
        if !host.is_empty() {
            ssh_key.host = host;
        }

        let port = cli_get_input("Port: ", "");
        if !port.is_empty() {
            ssh_key.port = port
                .parse::<u16>()
                .map_err(|_| CliError::Generic("Invalid port number".to_string()))?;
        }

        let username = cli_get_input("Username: ", "");
        if !username.is_empty() {
            ssh_key.username = username.to_string();
        }

        let password = cli_get_password("Password (optional): ", true);
        if !password.is_empty() {
            ssh_key.password = password.to_string();
        }

        let notes = cli_get_multiline_input("Notes");
        if !notes.is_empty() {
            ssh_key.notes = notes.to_string();
        }

        let ssh_key_str = serde_json::to_string(&ssh_key)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Edit item
        rpc::send::<&String, bool>("ssh.edit", &vec![&req.args[0].to_lowercase(), &ssh_key_str])?;

        cli_info!("Updated entry info for {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Edit SSH key",
            "nyx ssh edit <NAME>",
            "Edit details of an SSH key",
        );

        help.add_param("NAME", "Name of the entry to edit.");
        help.add_example("nyx ssh edit mysite/cloudflare");
        help
    }
}
