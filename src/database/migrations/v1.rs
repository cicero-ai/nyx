// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use crate::database::nyxdb::{MAGIC_BYTES, VERSION};
use crate::database::{FilesDb, HistoryDb, NotesDb, NyxDb, OtpDb, SshKeysDb, StringsDb, UsersDb};
use bincode::{Decode, Encode, config};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Migrate
pub fn migrate(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    // Decode
    let (old, _len): (NyxDbV1, usize) = bincode::decode_from_slice(&bytes[5..], config::standard())
        .map_err(|e| Error::Db(format!("Unable to load database: {}", e)))?;

    let db = NyxDb {
        default_timeout: old.default_timeout.into(),
        users: old.users.into(),
        otp: old.oauth.into(),
        ssh_keys: old.ssh_keys.into(),
        strings: old.strings.into(),
        notes: old.notes.into(),
        files: FilesDb::default(),
        history: old.history.into(),
    };

    // Encode via bincode
    let encoded: Vec<u8> = bincode::encode_to_vec(&db, config::standard())
        .map_err(|e| Error::Db(format!("Unable to save database: {}", e)))?;

    // Get output
    let mut output = vec![];
    output.extend_from_slice(MAGIC_BYTES);
    output.push(VERSION);
    output.extend(encoded);

    Ok(output)
}

#[derive(Default, Encode, Decode)]
pub struct NyxDbV1 {
    pub default_timeout: DatabaseTimeoutV1,
    pub users: UsersDbV1,
    pub oauth: OauthDbV1,
    pub ssh_keys: SshKeysDbV1,
    pub strings: StringsDbV1,
    pub notes: NotesDbV1,
    pub history: HistoryDbV1,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Decode, Encode)]
pub enum DatabaseTimeoutV1 {
    #[default]
    Never,
    Duration(Duration),
}

#[derive(Default, Decode, Encode)]
pub struct HistoryDbV1(pub Vec<HistoryItemV1>);

#[derive(Clone, Decode, Encode, Serialize, Deserialize)]
pub struct HistoryItemV1 {
    pub action: HistoryActionV1,
    pub data_type: HistoryDataTypeV1,
    pub source: String,
    pub dest: String,
    pub timestamp: u64,
}

#[derive(Decode, Encode, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum HistoryActionV1 {
    Create,
    Update,
    Delete,
    Copy,
    Rename,
}

#[derive(Decode, Encode, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum HistoryDataTypeV1 {
    User,
    Otp,
    SshKey,
    StrItem,
    Note,
}

#[derive(Default, Encode, Decode)]
pub struct NotesDbV1(pub HashMap<String, NoteV1>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct NoteV1 {
    pub display_name: String,
    pub note: String,
}

#[derive(Default, Encode, Decode)]
pub struct OauthDbV1(pub HashMap<String, OauthV1>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct OauthV1 {
    pub display_name: String,
    pub secret_code: String,
    pub url: String,
    pub recovery_keys: String,
}

#[derive(Default, Encode, Decode)]
pub struct SshKeysDbV1 {
    pub files: HashMap<String, SshKeyV1>,
    pub directories: HashMap<String, u64>,
    pub ino2name: HashMap<u64, SshFsEntryV1>,
}

#[derive(Default, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct SshKeyV1 {
    pub display_name: String,
    pub ino: u64,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub public_key: String,
    pub private_key: Vec<u8>,
    pub notes: String,
}

#[derive(Decode, Encode, Eq, PartialEq, Hash)]
pub struct SshFsEntryV1 {
    pub is_directory: bool,
    pub name: String,
}

#[derive(Default, Encode, Decode)]

pub struct StringsDbV1(pub HashMap<String, StrItemV1>);

#[derive(Clone, Decode, Encode, Serialize, Deserialize)]
pub struct StrItemV1 {
    pub display_name: String,
    pub value: String,
}

#[derive(Default, Encode, Decode)]
pub struct UsersDbV1(pub HashMap<String, UserV1>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct UserV1 {
    pub display_name: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

impl From<DatabaseTimeoutV1> for crate::database::DatabaseTimeout {
    fn from(v: DatabaseTimeoutV1) -> Self {
        match v {
            DatabaseTimeoutV1::Never => Self::Never,
            DatabaseTimeoutV1::Duration(d) => Self::Duration(d),
        }
    }
}

impl From<HistoryActionV1> for crate::database::HistoryAction {
    fn from(v: HistoryActionV1) -> Self {
        match v {
            HistoryActionV1::Create => Self::Create,
            HistoryActionV1::Update => Self::Update,
            HistoryActionV1::Delete => Self::Delete,
            HistoryActionV1::Copy => Self::Copy,
            HistoryActionV1::Rename => Self::Rename,
        }
    }
}

impl From<HistoryDataTypeV1> for crate::database::HistoryDataType {
    fn from(v: HistoryDataTypeV1) -> Self {
        match v {
            HistoryDataTypeV1::User => Self::User,
            HistoryDataTypeV1::Otp => Self::Otp,
            HistoryDataTypeV1::SshKey => Self::SshKey,
            HistoryDataTypeV1::StrItem => Self::StrItem,
            HistoryDataTypeV1::Note => Self::Note,
        }
    }
}

impl From<HistoryItemV1> for crate::database::HistoryItem {
    fn from(v: HistoryItemV1) -> Self {
        Self {
            action: v.action.into(),
            data_type: v.data_type.into(),
            source: v.source,
            dest: v.dest,
            timestamp: v.timestamp,
        }
    }
}

impl From<HistoryDbV1> for HistoryDb {
    fn from(v: HistoryDbV1) -> Self {
        Self(v.0.into_iter().map(Into::into).collect())
    }
}

impl From<NoteV1> for crate::database::Note {
    fn from(v: NoteV1) -> Self {
        Self {
            display_name: v.display_name,
            note: v.note,
        }
    }
}

impl From<NotesDbV1> for NotesDb {
    fn from(v: NotesDbV1) -> Self {
        Self(v.0.into_iter().map(|(k, val)| (k, val.into())).collect())
    }
}

impl From<OauthV1> for crate::database::Otp {
    fn from(v: OauthV1) -> Self {
        Self {
            display_name: v.display_name,
            secret_code: v.secret_code,
            url: v.url,
            recovery_keys: v.recovery_keys,
        }
    }
}

impl From<OauthDbV1> for OtpDb {
    fn from(v: OauthDbV1) -> Self {
        Self(v.0.into_iter().map(|(k, val)| (k, val.into())).collect())
    }
}

impl From<SshKeyV1> for crate::database::SshKey {
    fn from(v: SshKeyV1) -> Self {
        Self {
            display_name: v.display_name,
            host: v.host,
            port: v.port,
            username: v.username,
            password: v.password,
            public_key: v.public_key,
            private_key: v.private_key,
            notes: v.notes,
        }
    }
}

impl From<SshKeysDbV1> for SshKeysDb {
    fn from(v: SshKeysDbV1) -> Self {
        Self(v.files.into_iter().map(|(k, val)| (k, val.into())).collect())
    }
}

impl From<StrItemV1> for crate::database::StrItem {
    fn from(v: StrItemV1) -> Self {
        Self {
            display_name: v.display_name,
            value: v.value,
        }
    }
}

impl From<StringsDbV1> for StringsDb {
    fn from(v: StringsDbV1) -> Self {
        Self(v.0.into_iter().map(|(k, val)| (k, val.into())).collect())
    }
}

impl From<UserV1> for crate::database::User {
    fn from(v: UserV1) -> Self {
        Self {
            display_name: v.display_name,
            username: v.username,
            password: v.password,
            url: v.url,
            notes: v.notes,
        }
    }
}

impl From<UsersDbV1> for UsersDb {
    fn from(v: UsersDbV1) -> Self {
        Self(v.0.into_iter().map(|(k, val)| (k, val.into())).collect())
    }
}
