// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::cli;
use crate::database::SshKey;
use crate::error::Error;
use crate::rpc;
use argon2::{Algorithm, Argon2, Params, Version};
use falcon_cli::*;
use rand::rngs::OsRng;
use ssh_key::private::Ed25519Keypair;
use ssh_key::private::Ed25519PrivateKey;
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

        // Generate key
        let privkey = if req.has_flag("--seed") {
            self.generate_deterministic_key()?
        } else {
            let ed25519_keypair = Ed25519Keypair::random(&mut OsRng);
            PrivateKey::from(ed25519_keypair)
        };

        let private_key = privkey
            .to_openssh(LineEnding::LF)
            .map_err(|e| Error::Validate(format!("Unable to convert SSH key to OpenSSH format: {}", e)))?;

        let public_key = privkey
            .public_key()
            .to_openssh()
            .map_err(|e| Error::Validate(format!("Unable to convert private SSH key to public: {}", e)))?;

        // Instantiate item
        let ssh_key = SshKey {
            display_name: req.args[0].to_string(),
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
        if let Err(e) = rpc::send::<&String, bool>("ssh.import", &vec![&req.args[0].to_lowercase(), &key_str]) {
            return Err(Error::Generic(format!("Unable to create new SSH key: {}", e)).into());
        }

        cli_info!("Created new entry, {}", req.args[0]);

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "Generate SSH Key",
            "nyx ssh gen <NAME> [--seed]",
            "Generates new SSH key.",
        );

        help.add_param("NAME", "Name of SSH key to generate.");
        help.add_flag("--seed", "If present, will prompt for pass phrase / seed to generate determinisitc SSH key that can be generated again using same seed.");
        help.add_example("nyx ssh gen mysite/cloudflare");
        help
    }
}

impl CliSshKeyGenerate {
    fn generate_deterministic_key(&self) -> Result<PrivateKey, Error> {
        cli_sendln!(
            "Enter the pass phrase / seed to generate the SSH key with.  In the future, you may re-generate the exact same SSH key using this pass phrase / seed.\n"
        );
        let passphrase = cli_get_password("Passphrase: ", false);
        let salt = "NyxPass_1.0";

        // Configure Argon2id — tune these for your threat model
        let params = Params::new(64 * 1024, 3, 1, Some(32))
            .map_err(|e| Error::Validate(format!("Argon2 params error: {}", e)))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        // Derive 32 bytes of key material
        let mut seed = [0u8; 32];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt.as_bytes(), &mut seed)
            .map_err(|e| Error::Validate(format!("Key derivation failed: {}", e)))?;

        // Build Ed25519 keypair directly from the seed bytes
        let private = Ed25519PrivateKey::from_bytes(&seed);
        let keypair = Ed25519Keypair::from(private);
        let privkey = PrivateKey::from(keypair);

        Ok(privkey)
    }
}
