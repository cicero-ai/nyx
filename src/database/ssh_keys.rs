// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{BaseDbFunctions, BaseDbItem};
use crate::Error;
use crate::rpc::{CmdResponse, message};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Default, Encode, Decode, Serialize, Deserialize)]
pub struct SshKeysDb(pub HashMap<String, SshKey>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SshKey {
    pub display_name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub public_key: String,
    pub private_key: Vec<u8>,
    pub notes: String,
}

impl SshKeysDb {
    /// Generate ssh key
    pub fn generate(&mut self, req_id: usize, _params: &Vec<String>) -> Result<CmdResponse, Error> {
        Ok(CmdResponse::none(message::ok(req_id, true)))
    }

    /// Copy item
    pub fn copy_key(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.len() < 2 {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if self.contains_key(&params[1].to_lowercase()) {
            return Err(Error::Validate(format!(
                "Destination to copy item to already exists, {}",
                params[1]
            )));
        }

        // Get item
        let item = self.get(&params[0].to_lowercase()).ok_or(Error::Validate(format!(
            "Entry to copy  does not exist at, {}",
            params[0]
        )))?;

        // Copy
        let mut new_item = item.clone();
        new_item.set_name(&params[1]);

        // Insert
        self.insert(params[1].to_lowercase(), new_item);

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }
    /// Delete item
    pub fn delete_key(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if !self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!("No entry to delete exists at {}", params[0])));
        }

        // Delete
        self.remove(&params[0].to_lowercase());
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Import key
    pub fn import(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }
        let item: SshKey = serde_json::from_str(&params[1])?;

        // Check if exists
        if self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!("Entry already exists, {}", params[0])));
        }

        // Insert
        self.insert(params[0].to_lowercase(), item);
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Rename item
    pub fn rename_key(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.len() < 2 {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if self.contains_key(&params[1].to_lowercase()) {
            return Err(Error::Validate(format!(
                "Destination to rename item to already exists, {}",
                params[1]
            )));
        }

        // Get item
        let item = self
            .get(&params[0].to_lowercase())
            .ok_or(Error::Validate(format!("No entry exists at, {}", params[0])))?
            .clone();

        // Rename
        let mut new_item = item.clone();
        new_item.set_name(&params[1]);

        // Insert
        self.insert(params[1].to_lowercase(), new_item);
        self.remove(&params[0].to_lowercase());

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }
}

impl BaseDbFunctions for SshKeysDb {
    type Item = SshKey;
}

impl BaseDbItem for SshKey {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search) || self.host.to_lowercase().contains(search)
    }
}

impl Deref for SshKeysDb {
    type Target = HashMap<String, SshKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SshKeysDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Zeroize for SshKeysDb {
    fn zeroize(&mut self) {
        for (mut k, mut v) in self.0.drain() {
            k.zeroize();
            v.zeroize();
        }
        self.0.shrink_to_fit();
    }
}

impl Drop for SshKeysDb {
    fn drop(&mut self) {
        self.zeroize();
    }
}
