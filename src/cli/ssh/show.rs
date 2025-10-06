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
pub struct CliSshKeyShow {}

impl CliCommand for CliSshKeyShow {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx ssh show <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if item already exists
        cli::check_exists("ssh", &req.args[0], true)?;

        // Get item
        let ssh_key: SshKey = rpc::send("ssh.get", &vec![&req.args[0]])?;

        // Get vector
        let data = indexmap! {
            "Host:" => ssh_key.host.to_string(),
            "Port:" => format!("{}", ssh_key.port),
            "username:" => ssh_key.username.to_string(),
            "Password:" => ssh_key.password.to_string(),
            "Notes:" => ssh_key.notes.to_string()
        };

        // Show item info
        cli_header(&format!("SSH Key: {}", req.args[0]));
        cli_display_array(&data);
        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Show SSH Key Details",
            "nyx ssh show <NAME>",
            "Displays all details on a SSH key",
        );

        help.add_param("NAME", "Name of entry to show details of.");
        help.add_example("nyx ssh show mysite/cloudflare");
        help
    }
}
