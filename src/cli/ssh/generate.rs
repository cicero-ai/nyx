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
use rand::rngs::OsRng;
use rsa::RsaPrivateKey;
use ssh_key::private::RsaKeypair;
use ssh_key::{LineEnding, PrivateKey};

#[derive(Default)]
pub struct CliSshKeyGenerate {}

impl CliCommand for CliSshKeyGenerate {
    fn process(&self, req: &CliRequest) -> anyhow::Result<()> {
        // Check params
        if req.args.is_empty() {
            cli_error!("You did not specify a name of an entry");
            cli_info!("    Usage: nyx ssh gen <NAME>\n");
            return Err(CliError::MissingParams.into());
        }

        // Check if entry already exists
        cli::check_exists("ssh", &req.args[0], false)?;

        // Get info
        cli_header("Generate SSH Key");
        cli_info!("Enter the new SSH key information below.  Leave blank to omit a field.\n");
        let host = cli_get_input("Host: ", "");
        let port = cli_get_input("Port [22]: ", "22");
        let username = cli_get_input("Username [root]: ", "root");
        let password = cli_get_password("Password (optional): ", true);
        let notes = cli_get_multiline_input("Notes");

        cli_send!("Generating 4096 bit private key, please be patient... ");
        // Generate
        let rsa_key = RsaPrivateKey::new(&mut OsRng, 4096)
            .map_err(|e| Error::Validate(format!("Unable to generate RSA key: {}", e)))?;
        let rsa_keypair = RsaKeypair::try_from(rsa_key)
            .map_err(|e| Error::Validate(format!("Unable to convert to RsaKeypair: {}", e)))?;

        let privkey = PrivateKey::from(rsa_keypair);
        let private_key = privkey.to_openssh(LineEnding::LF).map_err(|e| {
            Error::Validate(format!(
                "Unable to convert SSH key to OpenSSH format: {}",
                e
            ))
        })?;
        let public_key = privkey.public_key().to_openssh().map_err(|e| {
            Error::Validate(format!(
                "Unable to convert private SSH key to public: {}",
                e
            ))
        })?;
        cli_send!(" done\n");

        // Instantiate item
        let ssh_key = SshKey {
            display_name: req.args[0].to_string(),
            ino: 0,
            host,
            port: port.parse::<u16>()?,
            username,
            password,
            public_key,
            private_key: private_key.as_bytes().to_vec(),
            notes,
        };

        let key_str = serde_json::to_string(&ssh_key)
            .map_err(|e| CliError::Generic(format!("Unable to serialize JSON object: {}", e)))?;

        // Create item
        if let Err(e) =
            rpc::send::<&String, bool>("ssh.import", &vec![&req.args[0].to_lowercase(), &key_str])
        {
            return Err(Error::Generic(format!("Unable to create new SSH key: {}", e)).into());
        }

        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Generate SSH Key",
            "nyx ssh gen <NAME>",
            "Generates new 4096 bit RSA SSH key.",
        );

        help.add_param("NAME", "Name of SSH key to generate.");
        help.add_example("nyx ssh gen mysite/cloudflare");
        help
    }
}
