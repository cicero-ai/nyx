// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{NyxDb, SshKeysDb};
use fuser::{
    FileType, Filesystem, KernelConfig, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    ReplyOpen, Request,
};
use libc::ENOENT;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const INO_ROOT: u64 = 1;
pub const TTL: Duration = Duration::from_secs(1);

pub struct NyxFs(pub Arc<Mutex<NyxDb>>);

impl Filesystem for NyxFs {
    fn init(&mut self, _req: &Request, _config: &mut KernelConfig) -> Result<(), libc::c_int> {
        Ok(())
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        self.0.lock().unwrap().ssh_keys.lookup(_req, parent, name, reply)
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply_ino: Option<u64>, reply: ReplyAttr) {
        self.0.lock().unwrap().ssh_keys.getattr(_req, ino, reply_ino, reply)
    }

    fn open(&mut self, _req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        self.0.lock().unwrap().ssh_keys.open(_req, ino, _flags, reply)
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        self.0.lock().unwrap().ssh_keys.read(
            _req,
            ino,
            _fh,
            offset,
            size,
            flags,
            _lock_owner,
            reply,
        )
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, reply: ReplyDirectory) {
        self.0.lock().unwrap().ssh_keys.readdir(_req, ino, _fh, offset, reply)
    }
}

impl Filesystem for SshKeysDb {
    fn init(&mut self, _req: &Request, _config: &mut KernelConfig) -> Result<(), libc::c_int> {
        Ok(())
    }

    // Lookup directory by name
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // Get directory name
        let mut name_str = match name.to_str() {
            Some(r) => r.to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        // Add parent dir, if needed
        if let Some(parent_entry) = self.ino2name.get(&parent) {
            name_str = format!("{}/{}", parent_entry.name, name_str);
        }

        // Check for root /ssh_keys/
        if name_str == "ssh_keys" {
            let attr = self.get_attr(2).unwrap();
            reply.entry(&TTL, &attr, 0);
            return;
        }
        name_str = name_str.trim_start_matches("ssh_keys").trim_start_matches("/").to_string();

        // Lookup info of directory / file
        let ino = match self.directories.get(&name_str) {
            Some(r) => *r,
            None => match self.get(&name_str) {
                Some(key) => key.ino,
                None => {
                    reply.error(ENOENT);
                    return;
                }
            },
        };

        if let Some(attr) = self.get_attr(ino) {
            reply.entry(&TTL, &attr, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    /// Get file attributes
    fn getattr(&mut self, _req: &Request, ino: u64, _reply_ino: Option<u64>, reply: ReplyAttr) {
        if let Some(attr) = self.get_attr(ino) {
            reply.attr(&TTL, &attr);
        } else {
            reply.error(ENOENT);
        }
    }

    /// Open a file
    fn open(&mut self, _req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        if self.ino2name.contains_key(&ino) {
            reply.opened(0, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    // Read an opened file
    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        // Get filename
        if let Some(fs_entry) = self.ino2name.get(&ino)
            && let Some(key) = self.files.get(&fs_entry.name)
        {
            let start = offset as usize;
            let end = (offset as usize + size as usize).min(key.private_key.len());

            if start < key.private_key.len() {
                reply.data(&key.private_key[start..end]);
            } else {
                reply.data(&[]);
            }
            return;
        }

        reply.error(ENOENT);
    }

    /// Read directory  entries
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        // Get directory name
        let fs_entry = match self.ino2name.get(&ino) {
            Some(r) => r,
            None => {
                reply.error(ENOENT);
                return;
            }
        };
        let dirname = if fs_entry.name.is_empty() {
            "".to_string()
        } else {
            format!("{}/", fs_entry.name)
        };

        // Root entries
        let mut entries = vec![
            (INO_ROOT, FileType::Directory, Path::new(".")),
            (INO_ROOT, FileType::Directory, Path::new("..")),
        ];

        // Add directories
        if ino == 1 {
            entries.push((2, FileType::Directory, Path::new("ssh_keys")));
        } else {
            let mut added: HashSet<String> = HashSet::new();
            for (name, key) in self.files.iter() {
                if !name.starts_with(&dirname) {
                    continue;
                }
                if let Some(short_name) = name.trim_start_matches(&dirname).split("/").next()
                    && !added.contains(&short_name.to_string())
                {
                    added.insert(short_name.to_string());
                    entries.push((key.ino, FileType::RegularFile, Path::new(short_name)));
                }
            }
        }

        for (i, (ino, kind, name)) in entries.into_iter().enumerate().skip(offset as usize) {
            let next_offset = (i + 1) as i64;
            if reply.add(ino, next_offset, kind, name) {
                break;
            }
        }

        reply.ok();
    }
}
