// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use std::sync::{Arc, Mutex};
use falcon_cli::*;
use tokio::task;
use rsa::{pkcs1v15::SigningKey, signature::RandomizedSigner, RsaPrivateKey};
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use sha2::{Sha256, Sha512};
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::SignRequest;
use ssh_agent_lib::async_trait;
use ssh_agent_lib::proto::Identity;
use ssh_key::{PrivateKey, PublicKey, HashAlg, Signature};
use ssh_key::private::KeypairData;
use crate::database::NyxDb;
use crate::Error;

#[derive(Clone)]
pub struct SshAgentDaemon {
    nyxdb: Arc<Mutex<NyxDb>>,
}

impl SshAgentDaemon {

    pub async fn start(nyxdb: &Arc<Mutex<NyxDb>>) -> Result<(), Error> {

        let agent = Self {
            nyxdb: Arc::clone(nyxdb)
        };

        #[cfg(unix)]
        {
            let agent_socket_path = std::env::var("SSH_AUTH_SOCK")
                .unwrap_or_else(|_| "/tmp/nyx-agent.sock".to_string());
            let _ = std::fs::remove_file(&agent_socket_path);
            let listener = tokio::net::UnixListener::bind(&agent_socket_path)?;
            cli_info!("SSH agent listening on {}", agent_socket_path);

            task::spawn(async move {
                if let Err(e) = ssh_agent_lib::agent::listen(listener, agent).await {
                    cli_error!("SSH agent listener error: {}", e);
                }
            });
        }

        #[cfg(windows)]
        {
            let pipe_name = std::env::var("SSH_AUTH_SOCK")
                .unwrap_or_else(|_| r"\\.\pipe\nyx-agent".to_string());
            let listener = ssh_agent_lib::agent::NamedPipeListener::bind(&pipe_name)?;
            cli_info!("SSH agent listening on {}", pipe_name);

            task::spawn(async move {
                if let Err(e) = ssh_agent_lib::agent::listen(listener, agent).await {
                    cli_error!("SSH agent listener error: {}", e);
                }
            });
        }

        Ok(())
    }

    fn find_private_key(&self, request: &SignRequest) -> Result<PrivateKey, AgentError> {

        let db = match self.nyxdb.lock() {
            Ok(r) => r,
            Err(_) => return Err(AgentError::other(crate::Error::Generic("Unable to lock database".to_string())))
        };

        let key = match db.ssh_keys.values().find(|key| {
            if let Ok(public_key) = PublicKey::from_openssh(&key.public_key) {
                *public_key.key_data() == request.pubkey
            } else { false }
        }) {
            Some(r) => r,
            None => return Err(AgentError::other(crate::Error::Generic("No key found".to_string())))
        };

        let privkey = PrivateKey::from_openssh(&key.private_key)
            .map_err(AgentError::other)?;

        Ok(privkey)
    }

    fn determine_algorithm(&self, privkey: &PrivateKey, request: &SignRequest) -> Result<(HashAlg, String), AgentError> {

        // Get algorithm
        let alg = match privkey.key_data() {
            KeypairData::Rsa(_) => {
                if request.flags & 0x02 != 0 { 
                    HashAlg::Sha256
                } else if request.flags & 0x04 != 0 { // SSH_AGENT_RSA_SHA2_512
                    HashAlg::Sha512
                } else {
                    HashAlg::Sha256
                }
            },
            _ => HashAlg::Sha256,
        };

        let alg_name = match alg {
            HashAlg::Sha256 => "rsa-sha2-256",
            HashAlg::Sha512 => "rsa-sha2-512",
            _ => "ssh-rsa",
        };

        Ok((alg, alg_name.to_string()))
    }

    fn sign_rsa(&self, privkey: &PrivateKey, request: &SignRequest, alg: HashAlg) -> Result<Signature, AgentError> {

        // Extract RSA key data
        let KeypairData::Rsa(rsa_keypair) = privkey.key_data() else {
            return Err(AgentError::other(Error::Generic("not an RSA key".to_string())));
        };

        // Manually construct RsaPrivateKey from components to work around
        // upstream bug in ssh-key 0.6.7 where TryFrom<&RsaKeypair> passes [p, p]
        // instead of [p, q] as the prime factors.
        let rsa_privkey = RsaPrivateKey::from_components(
            rsa::BigUint::try_from(&rsa_keypair.public.n).map_err(AgentError::other)?,
            rsa::BigUint::try_from(&rsa_keypair.public.e).map_err(AgentError::other)?,
            rsa::BigUint::try_from(&rsa_keypair.private.d).map_err(AgentError::other)?,
            vec![
                rsa::BigUint::try_from(&rsa_keypair.private.p).map_err(AgentError::other)?,
                rsa::BigUint::try_from(&rsa_keypair.private.q).map_err(AgentError::other)?,
            ],
        ).map_err(AgentError::other)?;

        // Sign the request
        let mut rng = rand::thread_rng();
        let sig_bytes: Vec<u8> = match alg {
            HashAlg::Sha256 => {
                let signing_key = SigningKey::<Sha256>::new(rsa_privkey);
                signing_key.sign_with_rng(&mut rng, &request.data).to_bytes().into()
            }
            HashAlg::Sha512 => {
                let signing_key = SigningKey::<Sha512>::new(rsa_privkey);
                signing_key.sign_with_rng(&mut rng, &request.data).to_bytes().into()
            }
            _ => return Err(AgentError::other(Error::Generic("unsupported hash".to_string()))),
        };

        // Raw bytes back to SSH sig
        let sig = Signature::new(
            ssh_key::Algorithm::Rsa { hash: Some(alg) },
            sig_bytes,
        ).map_err(AgentError::other)?;

        Ok(sig)
    }

}

#[async_trait]
impl Session for SshAgentDaemon {
    /// Return all public SSH keys supported by this ssh-agent
    async fn request_identities(&mut self) -> Result<Vec<Identity>, AgentError> {

        let db = match self.nyxdb.lock() {
            Ok(r) => r,
            Err(_) => return Err(AgentError::other(crate::Error::Generic("Unable to lock database".to_string())))
        };

        let keys: Vec<Identity> = db.ssh_keys.iter().filter_map(|(name, key)| {
            if let Ok(public_key) = PublicKey::from_openssh(&key.public_key) {
                Some(Identity {
                    pubkey: public_key.key_data().clone(),
                    comment: name.to_string(),
                })
            } else { 
                None
            }
        }).collect();

        Ok(keys)
    }

    async fn sign(&mut self, request: SignRequest) -> Result<Signature, AgentError> {

        // Get private key
        let privkey = self.find_private_key(&request)?;

        // Determine algorithm to use
        let (alg, _alg_name) = self.determine_algorithm(&privkey, &request)?;

        // Sign the request.
        // For RSA we must manually construct the key to work around an upstream
        // ssh-key 0.6.7 bug, and to respect the hash algorithm from request flags.
        // For Ed25519/ECDSA we use the Signer trait which signs the raw data directly.
        // NOTE: Do NOT use PrivateKey::sign(namespace, hash_alg, msg) here — that
        // produces an SshSig envelope (PROTOCOL.sshsig format), not the raw signature
        // that the SSH agent protocol expects.
        let sig: ssh_key::Signature = match privkey.key_data() {
            KeypairData::Rsa(_) => self.sign_rsa(&privkey, &request, alg)?,
            _ => {
                Signer::try_sign(&privkey, &request.data)
                    .map_err(AgentError::other)?
            }
        };

        Ok(sig)
    }
}


