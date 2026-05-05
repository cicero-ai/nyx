// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{BaseDbFunctions, HistoryDb, NotesDb, OtpDb, SshKeysDb, StringsDb, UsersDb, FilesDb};
use crate::Error;
use crate::security::SecureBuffer;
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

pub const MAGIC_BYTES: &[u8; 4] = b"NYX\0";
pub const VERSION: u8 = 2;

#[derive(Default, Encode, Decode)]
pub struct NyxDb {
    pub default_timeout: DatabaseTimeout,
    pub users: UsersDb,
    pub otp: OtpDb,
    pub ssh_keys: SshKeysDb,
    pub strings: StringsDb,
    pub notes: NotesDb,
    pub files: FilesDb,
    pub history: HistoryDb,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Decode, Encode)]
pub enum DatabaseTimeout {
    #[default]
    Never,
    Duration(Duration),
}

#[derive(Serialize, Deserialize)]
pub struct DbStats {
    pub dbfile: String,
    pub users: (u32, u32),
    pub otp: (u32, u32),
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
            users: UsersDb::default(),
            otp: OtpDb::default(),
            ssh_keys: SshKeysDb::default(),
            notes: NotesDb::default(),
            strings: StringsDb::default(),
            files: FilesDb::default(),
            history: HistoryDb::default()
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
        // Encode via bincode into a SecureBuffer (mlock'd, excluded from dumps)
        let encoded: Vec<u8> = bincode::encode_to_vec(&*self, config::standard())
            .map_err(|e| Error::Db(format!("Unable to save database: {}", e)))?;

        let mut output_vec = Vec::with_capacity(5 + encoded.len());
        output_vec.extend_from_slice(MAGIC_BYTES);
        output_vec.push(VERSION);
        output_vec.extend(encoded);

        // Wrap plaintext in SecureBuffer -- locked in RAM, zeroized on drop
        let output = SecureBuffer::from_vec(output_vec)?;

        // Resave file if just updating
        if Path::new(&dbfile).exists() && master_key.is_none() {
            crypto::update_existing_file(dbfile, output.as_slice(), n_password)?;
            return Ok(());
        }

        // Encrypt bytes
        let encrypted = if let Some(m_key) = master_key {
            crypto::encrypt_with_master_key(output.as_slice(), n_password, m_key)?
        } else {
            crypto::encrypt(output.as_slice(), n_password)?
        };

        // Check parent dir
        if let Some(parent) = Path::new(&dbfile).parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        // Save file
        fs::write(dbfile, &encrypted)?;

        // output drops here: zeroized + munlock'd
        Ok(())
    }

    /// Load database from file
    pub fn load(dbfile: &str, n_password: [u8; 32]) -> Result<Self, Error> {
        // Read file
        let encrypted_bytes = fs::read(dbfile)?;

        // Decrypt into a SecureBuffer -- locked in RAM, excluded from core dumps
        let decrypted = crypto::decrypt(&encrypted_bytes, n_password)?;
        let mut bytes = SecureBuffer::from_vec(decrypted)?;
        if !bytes.starts_with(MAGIC_BYTES) {
            return Err(Error::Db("Not a valid Nyx database file.".to_string()));
        }

        // Migrate if needed
        if bytes[4] == 1 {
            let migrated = crate::database::migrations::v1::migrate(bytes.as_slice())?;
            bytes = SecureBuffer::from_vec(migrated)?;
            crypto::update_existing_file(dbfile, bytes.as_slice(), n_password)?;
        }

        // Decode
        let (mut db, _len): (NyxDb, usize) = bincode::decode_from_slice(&bytes[5..], config::standard())
                .map_err(|e| Error::Db(format!("Unable to load database: {}", e)))?;

        // Protect all files
        for (_, file) in db.files.iter_mut() {
            file.is_protected = true;
        }

        // bytes drops here: zeroized + munlock'd
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

            let decrypted = match crypto::decrypt(&encrypted_bytes, n_password) {
                Ok(r) => r,
                Err(_) => {
                    cli_info!("Invalid password, please double check and try again.\n");
                    continue;
                }
            };

            // Wrap in SecureBuffer so decrypted bytes are locked in RAM and zeroized on drop
            let data = SecureBuffer::from_vec(decrypted)?;

            // Check header
            if data.len() < 5 {
                return Err(Error::Db(
                    "This is not a valid Nyx database file.".to_string(),
                ));
            } else if &data.as_slice()[0..4] != MAGIC_BYTES {
                return Err(Error::Db(
                    "This is not a valid Nyx database file.".to_string(),
                ));
            } else if data[4] != VERSION && data[4] != 1 {
                return Err(Error::Db(
                    "This is not a valid Nyx database file.".to_string(),
                ));
            }
            // data drops here: zeroized + munlock'd
            break;
        }

        Ok(n_password)
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
            otp: Self::get_item(&nyxdb.otp),
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

impl Zeroize for NyxDb {
    fn zeroize(&mut self) {
        self.users.zeroize();
        self.otp.zeroize();
        self.ssh_keys.zeroize();
        self.notes.zeroize();
        self.strings.zeroize();
        self.files.zeroize();
        self.history.zeroize();
    }
}

impl Drop for NyxDb {
    fn drop(&mut self) {
        self.zeroize();
    }
}




