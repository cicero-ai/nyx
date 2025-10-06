// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliSshKeyDelete {}

impl CliCommand for CliSshKeyDelete {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name to delete.");
            cli_info!("    Usage: nyx ssh rm <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Ensure  source exists
        cli::check_exists("ssh", &req.args[0], true)?;

        // Delete item
        rpc::send::<String, bool>("ssh.delete", &vec![req.args[0].to_string()])?;

        cli_info!("Deleted entry {}\n", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help =
            CliHelpScreen::new("Delete SSH Key", "nyx ssh rm <NAME>", "Deletes an SSH key");

        help.add_param("NAME", "Name of the entry to delete.");
        help.add_example("nyx ssh rm mysite/cloudflare");
        help
    }
}
