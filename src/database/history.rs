// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use crate::rpc::{CmdResponse, message};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, Decode, Encode)]
pub struct HistoryDb(pub Vec<HistoryItem>);

#[derive(Clone, Decode, Encode, Serialize, Deserialize)]
pub struct HistoryItem {
    pub action: HistoryAction,
    pub data_type: HistoryDataType,
    pub source: String,
    pub dest: String,
    pub timestamp: u64,
}

#[derive(Decode, Encode, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum HistoryAction {
    Create,
    Update,
    Delete,
    Copy,
    Rename,
}

#[derive(Decode, Encode, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum HistoryDataType {
    User,
    Otp,
    SshKey,
    StrItem,
    Note,
}

impl HistoryDb {
    /// Add item
    pub fn add(
        &mut self,
        action: HistoryAction,
        data_type: HistoryDataType,
        source: &str,
        dest: &str,
    ) -> Result<(), Error> {
        if action == HistoryAction::Create && data_type == HistoryDataType::Otp {
            return Ok(());
        }

        self.insert(
            0,
            HistoryItem {
                action,
                data_type,
                source: source.to_string(),
                dest: dest.to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            },
        );

        Ok(())
    }

    /// List items
    pub fn list_items(
        &mut self,
        req_id: usize,
        params: &Vec<String>,
    ) -> Result<CmdResponse, Error> {
        let start = params[0]
            .parse::<usize>()
            .map_err(|e| Error::Validate(format!("Invalid start number: {}", e)))?;

        let end = (start + 25).min(self.len());

        let items = self[start..end].to_vec();
        Ok(CmdResponse::none(message::ok(req_id, items)))
    }
}

impl Deref for HistoryDb {
    type Target = Vec<HistoryItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HistoryDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for HistoryAction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "edit" => Ok(Self::Update),
            "copy" => Ok(Self::Copy),
            "delete" => Ok(Self::Delete),
            "new" | "import" | "generate"|"set" => Ok(Self::Create),
            "rename" => Ok(Self::Rename),
            _ => Err(Error::Validate(format!("No history action for: {}", s))),
        }
    }
}

impl FromStr for HistoryDataType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Self::User),
            "otp" => Ok(Self::Otp),
            "ssh" => Ok(Self::SshKey),
            "str" => Ok(Self::StrItem),
            "note" => Ok(Self::Note),
            _ => Err(Error::Validate(format!("No history data type for: {}", s))),
        }
    }
}

impl fmt::Display for HistoryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for HistoryDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
