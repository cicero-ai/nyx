// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{BaseDbFunctions, HistoryDb, NotesDb, OauthDb, SshKeysDb, StringsDb, UsersDb};
use crate::Error;
use crate::security::crypto;
use bincode::{Decode, Encode, config};
use falcon_cli::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use zeroize::Zeroize;

const MAGIC_BYTES: &[u8; 4] = b"NYX\0";
const VERSION: u8 = 1;

#[derive(Default, Encode, Decode)]
pub struct NyxDb {
    pub default_timeout: DatabaseTimeout,
    pub users: UsersDb,
    pub oauth: OauthDb,
    pub ssh_keys: SshKeysDb,
    pub strings: StringsDb,
    pub notes: NotesDb,
    pub history: HistoryDb,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Decode, Encode)]
pub enum DatabaseTimeout {
    #[default]
    Never,
    Duration(Duration),
}

#[derive(Serialize, Deserialize)]
pub struct DbStats {
    pub dbfile: String,
    pub users: (u32, u32),
    pub oauth: (u32, u32),
    pub ssh_keys: (u32, u32),
    pub strings: (u32, u32),
    pub notes: (u32, u32),
}

impl NyxDb {
    /// Create new database
    pub fn create(
        filename: &str,
        password: &str,
        default_timeout: DatabaseTimeout,
    ) -> Result<Self, Error> {
        let mut db = Self {
            default_timeout,
            ..Default::default()
        };

        // Save
        let n_password = crypto::normalize_password(password);
        db.save(filename, n_password, None)?;
        Ok(db)
    }

    /// Save data store
    pub fn save(
        &mut self,
        dbfile: &str,
        n_password: [u8; 32],
        master_key: Option<[u8; 32]>,
    ) -> Result<(), Error> {
        // Encode via bincode
        let encoded: Vec<u8> = bincode::encode_to_vec(&*self, config::standard())
            .map_err(|e| Error::Db(format!("Unable to save database: {}", e)))?;

        // Get output
        let mut output = vec![];
        output.extend_from_slice(MAGIC_BYTES);
        output.push(VERSION);
        output.extend(encoded);

        // Resave file if just updating
        if Path::new(&dbfile).exists() && master_key.is_none() {
            crypto::update_existing_file(dbfile, &output, n_password)?;
            return Ok(());
        }

        // Encrypt bytes

        let encrypted = if let Some(m_key) = master_key {
            crypto::encrypt_with_master_key(&output, n_password, m_key)?
        } else {
            crypto::encrypt(&output, n_password)?
        };

        // Check parent dir
        if let Some(parent) = Path::new(&dbfile).parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        // Save file
        fs::write(dbfile, &encrypted)?;

        Ok(())
    }

    /// Load database from file
    pub fn load(dbfile: &str, n_password: [u8; 32]) -> Result<Self, Error> {
        // Read file
        let encrypted_bytes = fs::read(dbfile)?;

        // Decrypt
        let bytes = crypto::decrypt(&encrypted_bytes, n_password)?;

        // Decode
        let (db, _len): (NyxDb, usize) =
            bincode::decode_from_slice(&bytes[5..], config::standard())
                .map_err(|e| Error::Db(format!("Unable to load database: {}", e)))?;

        Ok(db)
    }

    /// Unlock database, essentially just ensure password is correct before refreshing command to start daemon
    pub fn unlock(dbfile: &str) -> Result<[u8; 32], Error> {
        cli_info!("Opening Nyx database located at:");
        cli_info!("    {}\n", dbfile);

        // Read file
        let encrypted_bytes = fs::read(dbfile)?;

        // Get correct password
        let mut n_password: [u8; 32];
        loop {
            let mut password = cli_get_password("Password: ", false);
            n_password = crypto::normalize_password(&password);
            password.zeroize();

            let data = match crypto::decrypt(&encrypted_bytes, n_password) {
                Ok(r) => r,
                Err(_) => {
                    cli_info!("Invalid password, please double check and try again.\n");
                    continue;
                }
            };

            // Check header
            if data.len() < 5 {
                return Err(Error::Db(
                    "This is not a valid Nyx database file.".to_string(),
                ));
            } else if &data[0..4] != MAGIC_BYTES {
                return Err(Error::Db(
                    "This is not a valid Nyx database file.".to_string(),
                ));
            } else if data[4] != VERSION {
                return Err(Error::Db(
                    "This is not a valid Nyx database file.".to_string(),
                ));
            }
            break;
        }

        Ok(n_password)
    }

    /// Secure clear
    pub fn secure_clear(&mut self) {
        self.users.secure_clear();
        self.oauth.secure_clear();
        self.ssh_keys.secure_clear();
        self.strings.secure_clear();
        self.notes.secure_clear();
    }
}

impl FromStr for DatabaseTimeout {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let res = match value.to_lowercase().as_str() {
            "n" => Self::Never,
            _ => Self::parse_duration(&value.to_lowercase())?,
        };

        Ok(res)
    }
}

impl DatabaseTimeout {
    /// Parse duration (secs, mins, hours)
    pub fn parse_duration(value: &str) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::Generic("Invalid duration".to_string()));
        }

        let mut chars: Vec<char> = value.chars().collect();
        let interval = chars.pop().unwrap();

        // Base seconds
        let secs: u64 = match interval {
            's' => 1,
            'm' => 60,
            'h' => 3600,
            _ => return Err(Error::Generic("Invalid duration".to_string())),
        };

        let tmp_value = String::from_iter(chars);
        let length = match tmp_value.parse::<u64>() {
            Ok(r) => r,
            Err(_) => return Err(Error::Generic("Invalid duration".to_string())),
        };

        let duration = Duration::from_secs(secs * length);
        Ok(DatabaseTimeout::Duration(duration))
    }
}

impl DbStats {
    pub fn new(dbfile: &str, nyxdb: &NyxDb) -> Self {
        Self {
            dbfile: dbfile.to_string(),
            users: Self::get_item(&nyxdb.users),
            oauth: Self::get_item(&nyxdb.oauth),
            ssh_keys: Self::get_item(&nyxdb.ssh_keys),
            strings: Self::get_item(&nyxdb.strings),
            notes: Self::get_item(&nyxdb.notes),
        }
    }

    pub fn get_item<T>(db: &T) -> (u32, u32)
    where
        T: BaseDbFunctions,
    {
        let mut dirs: HashSet<String> = HashSet::new();
        for key in db.keys() {
            if !key.contains("/") {
                continue;
            }

            let mut parts: Vec<&str> = key.split("/").collect();
            parts.pop().unwrap();
            let dirname = parts.join("/").to_string();
            dirs.insert(dirname);
        }

        (db.len() as u32, dirs.len() as u32)
    }
}
