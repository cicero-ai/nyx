// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli::{self, clipboard};
use crate::database::SshKey;
use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliSshKeyXv {}

impl CliCommand for CliSshKeyXv {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx ssh xp <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if entry already exists
        cli::check_exists("ssh", &req.args[0], true)?;

        // Get entry
        let ssh_key: SshKey = rpc::send("ssh.get", &vec![&req.args[0], &"1".to_string()])?;

        // Copy to clipboard
        let privkey = String::from_utf8(ssh_key.private_key).map_err(|e| {
            CliError::Generic(format!("Unable to encode private key to UTF-8: {}", e))
        })?;
        clipboard::copy(&privkey)?;

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Copy SSH Private Key",
            "nyx ssh xv <NAME>",
            "Copy SSH private key to clipboard",
        );

        help.add_param("NAME", "Name of entry to copy from.");
        help.add_example("nyx ssh xv mysite/cloudflare");
        help
    }
}
