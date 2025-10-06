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
use zeroize::Zeroize;

#[cfg(any(target_os="linux", feature = "fuse"))]
use fuser::{FileAttr, FileType};
#[cfg(any(target_os="linux", feature = "fuse"))]
use std::time::{Duration, UNIX_EPOCH};

#[derive(Encode, Decode)]
pub struct SshKeysDb {
    pub files: HashMap<String, SshKey>,
    pub directories: HashMap<String, u64>,
    pub ino2name: HashMap<u64, SshFsEntry>,
}

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct SshKey {
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
pub struct SshFsEntry {
    pub is_directory: bool,
    pub name: String,
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
        let max_ino = self.ino2name.keys().max().unwrap_or(&2) + 1;

        // Copy
        let mut new_item = item.clone();
        new_item.set_name(&params[1]);
        new_item.ino = max_ino;

        // Insert
        self.insert(params[1].to_lowercase(), new_item);
        self.ino2name.insert(
            max_ino,
            SshFsEntry {
                is_directory: false,
                name: params[1].to_lowercase(),
            },
        );

        // Sync directories
        self.sync_directory(&params[0].to_lowercase());
        self.sync_directory(&params[1].to_lowercase());

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }
    /// Delete item
    pub fn delete_key(
        &mut self,
        req_id: usize,
        params: &Vec<String>,
    ) -> Result<CmdResponse, Error> {
        // Validate
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if !self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!(
                "No entry to delete exists at {}",
                params[0]
            )));
        }

        // Remove ino
        let ino = self.get(&params[0].to_lowercase()).unwrap().ino;
        self.ino2name.remove(&ino);

        // Sync directory
        self.sync_directory(&params[0].to_lowercase());

        // Delete
        self.remove(&params[0].to_lowercase());
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Import key
    pub fn import(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }
        let mut item: SshKey = serde_json::from_str(&params[1])?;

        // Check if exists
        if self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!(
                "Entry already exists, {}",
                params[0]
            )));
        }

        // Get max ino
        let max_ino = self.ino2name.keys().max().unwrap_or(&2) + 1;
        item.ino = max_ino;

        // Insert
        self.insert(params[0].to_lowercase(), item);
        self.ino2name.insert(
            max_ino,
            SshFsEntry {
                is_directory: false,
                name: params[0].to_lowercase(),
            },
        );
        self.sync_directory(&params[0].to_lowercase());

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Rename item
    pub fn rename_key(
        &mut self,
        req_id: usize,
        params: &Vec<String>,
    ) -> Result<CmdResponse, Error> {
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
            .ok_or(Error::Validate(format!(
                "No entry exists at, {}",
                params[0]
            )))?
            .clone();

        // Rename
        let mut new_item = item.clone();
        new_item.set_name(&params[1]);

        // Insert
        self.insert(params[1].to_lowercase(), new_item);
        self.ino2name.insert(
            item.ino,
            SshFsEntry {
                is_directory: false,
                name: params[1].to_lowercase(),
            },
        );
        self.remove(&params[0].to_lowercase());

        // Sync directories
        self.sync_directory(&params[0].to_lowercase());
        self.sync_directory(&params[1].to_lowercase());

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Sync directory modification with fuse filesystem
    fn sync_directory(&mut self, name: &str) {
        if !name.contains("/") {
            return;
        }

        let mut parts: Vec<String> = name.split("/").map(|p| p.to_string()).collect();
        parts.pop().unwrap();
        let dirname = parts.join("/").to_string();

        // Check if entry exists
        let search = format!("{}/", dirname);
        if self.keys().any(|chk| chk.starts_with(&search)) {
            if self.directories.contains_key(&dirname) {
                return;
            }

            let max_ino = self.ino2name.keys().max().unwrap_or(&2) + 1;
            self.directories.insert(dirname.to_string(), max_ino);
            self.ino2name.insert(
                max_ino,
                SshFsEntry {
                    is_directory: true,
                    name: dirname.to_string(),
                },
            );

        // Remove directory
        } else if let Some(dir_ino) = self.directories.get(&dirname) {
            self.ino2name.remove(dir_ino);
            self.directories.remove(&dirname);
        }
    }

    #[cfg(any(target_os="linux", feature = "fuse"))]
    /// Get attributes for file system
    pub fn get_attr(&self, ino: u64) -> Option<FileAttr> {
        let ts = UNIX_EPOCH + Duration::from_secs(1609459200); // Jan 1, 2021
        let fs_entry = self.ino2name.get(&ino)?;

        // Set defaultt, mutable attr
        let mut attr = FileAttr {
            ino,
            size: 0,
            blocks: 0,
            blksize: 4096,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        };

        // Directory
        if fs_entry.is_directory {
            if !fs_entry.name.is_empty() {
                attr.nlink = (3 + fs_entry.name.chars().filter(|c| *c == '/').count()) as u32;
            }

            return Some(attr);
        }

        // Regular file
        let key = self.get(&fs_entry.name.to_string())?;

        // Set attributes
        attr.size = key.private_key.len() as u64;
        attr.kind = FileType::RegularFile;
        attr.perm = 0o600;
        attr.nlink = 1;

        Some(attr)
    }
}

impl BaseDbFunctions for SshKeysDb {
    type Item = SshKey;

    /// Secure clear
    fn secure_clear(&mut self) {
        for (_, key) in self.iter_mut() {
            key.display_name.zeroize();
            key.host.zeroize();
            key.port.zeroize();
            key.username.zeroize();
            key.public_key.zeroize();
            key.private_key.zeroize();
            key.notes.zeroize();
        }
    }
}

impl BaseDbItem for SshKey {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search)
            || self.host.to_lowercase().contains(search)
    }
}

impl Deref for SshKeysDb {
    type Target = HashMap<String, SshKey>;

    fn deref(&self) -> &Self::Target {
        &self.files
    }
}

impl DerefMut for SshKeysDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.files
    }
}

impl Default for SshKeysDb {
    fn default() -> Self {
        let mut db = Self {
            files: HashMap::new(),
            directories: HashMap::new(),
            ino2name: HashMap::new(),
        };

        db.directories.insert("".to_string(), 1);
        db.directories.insert("ssh_keys".to_string(), 2);

        db.ino2name.insert(
            1,
            SshFsEntry {
                is_directory: true,
                name: "".to_string(),
            },
        );

        db.ino2name.insert(
            2,
            SshFsEntry {
                is_directory: true,
                name: "".to_string(),
            },
        );

        db
    }
}
