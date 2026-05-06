// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{BaseDbFunctions, BaseDbItem};
use crate::Error;
#[cfg(any(target_os = "linux", feature = "fuse"))]
use crate::rpc::fs_launcher::get_mount_dir;
use crate::rpc::{CmdResponse, RpcTimer, TIMERS, message};
use bincode::{Decode, Encode};
use chrono::Local;
use nix::unistd::{getgid, getuid};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(any(target_os = "linux", feature = "fuse"))]
use fuser::{FileAttr, FileType, Request};
#[cfg(any(target_os = "linux", feature = "fuse"))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MIN_FILES_PER_GROUP: usize = 3;
const MAX_FILES_PER_GROUP: usize = 12;

#[derive(Default, Encode, Decode, Serialize, Deserialize)]
pub struct FilesDb(pub HashMap<String, ProtectedFile>);

#[derive(Default, Clone, Encode, Decode, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct ProtectedFile {
    pub filename: String,
    pub file_hash: String,
    pub ino: u64,
    pub is_protected: bool,
    pub contents: Vec<u8>,
    pub whitelist: Vec<String>,
}

#[cfg(any(target_os = "linux", feature = "fuse"))]
impl FilesDb {
    /// Check whether or not process is allowed to open file
    pub fn is_allowed(&self, req: &Request, file: &ProtectedFile) -> bool {
        if !file.is_protected || file.whitelist.is_empty() {
            return true;
        }

        let exe = match fs::read_link(format!("/proc/{}/exe", req.pid())) {
            Ok(r) => r,
            Err(_) => return false,
        };

        if file.whitelist.iter().any(|b| Path::new(b) == exe) {
            return true;
        }

        self.log_unauthorized_attempt(&exe, req, file);
        false
    }

    // Log unaithized  attempt
    fn log_unauthorized_attempt(&self, exe: &PathBuf, request: &Request, file: &ProtectedFile) {
        let body = format!(
            "The binary {:?} tried to access the protected file {}",
            exe, file.filename
        );
        let _ = Notification::new().summary("Nyx - Unauthoeized File Access").body(&body).show();

        if let Some(mut log_path) = dirs::home_dir() {
            let time = Local::now();
            let log_line = format!(
                "[{}] {:?} (pid {}) tried  to open {}",
                time.format("%Y-%m-%d %H:%M:%S"),
                exe,
                request.pid(),
                file.filename
            );
            log_path.push("nyx_unauthorized_access.log");

            if let Ok(mut log_file) = OpenOptions::new().append(true).create(true).open(log_path) {
                let _ = writeln!(log_file, "{}", log_line);
            }
        }
    }

    /// Get attributes for file system
    pub fn get_attr(&self, ino: u64) -> Option<FileAttr> {
        let ts = UNIX_EPOCH + Duration::from_secs(1609459200); // Jan 1, 2021

        // Set attr
        let mut attr = FileAttr {
            ino,
            size: 0,
            blocks: 0,
            blksize: 4096,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::RegularFile,
            perm: 0o600,
            nlink: 1,
            uid: getuid().into(),
            gid: getgid().into(),
            rdev: 0,
            flags: 0,
        };

        if ino < 3 {
            if ino == 1 {
                attr.perm = 0o755;
                attr.nlink = 2;
                attr.kind = FileType::Directory;
            }
            return Some(attr);
        }

        let fs_entry = self.values().find(|file| file.ino == ino)?;
        attr.size = fs_entry.contents.len() as u64;

        Some(attr)
    }

    pub fn mount_status() -> FileAttr {
        let ts = UNIX_EPOCH + Duration::from_secs(1609459200); // Jan 1, 2021

        FileAttr {
            ino: 2,
            size: 0,
            blocks: 0,
            blksize: 4096,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::RegularFile,
            perm: 0o600,
            nlink: 1,
            uid: getuid().into(),
            gid: getgid().into(),
            rdev: 0,
            flags: 0,
        }
    }

    fn group_files(&self, paths: &[String]) -> Vec<String> {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();

        for path in paths {
            let key = self.find_best_parent(path, paths);
            let relative = path.strip_prefix(&key).unwrap_or(path).trim_start_matches('/').to_string();
            groups.entry(key).or_default().push(relative);
        }

        self.render_groups(&groups)
    }

    fn render_groups(&self, groups: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();

        let mut keys: Vec<&String> = groups.keys().collect();
        keys.sort();

        for dir in keys {
            res.push(format!("{}:", dir));
            let mut files = groups[dir].clone();
            files.sort();
            for file in files {
                res.push(format!("    {}", file));
            }
            res.push("".to_string());
        }

        res
    }

    /// Get parent of file path
    fn find_best_parent(&self, path: &str, all_paths: &[String]) -> String {
        let components: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        // We care about directory components only, so drop the filename (last element).
        let dir_components = if components.len() > 1 {
            &components[..components.len() - 1]
        } else {
            &components[..]
        };

        let mut best: Option<String> = None;

        // Try each prefix depth from deepest to shallowest.
        for depth in (1..=dir_components.len()).rev() {
            let candidate = format!("/{}", dir_components[..depth].join("/"));
            let count = self.count_paths_under(&candidate, all_paths);

            if (MIN_FILES_PER_GROUP..=MAX_FILES_PER_GROUP).contains(&count) {
                return candidate;
            }

            // Keep the deepest candidate that has at least MIN files,
            // as a fallback if nothing hits the sweet spot perfectly.
            if count >= MIN_FILES_PER_GROUP && best.is_none() {
                best = Some(candidate);
            }
        }

        // If we found a decent fallback, use it.
        if let Some(b) = best {
            return b;
        }

        // Last resort: use the immediate parent directory of this file.
        let fallback_depth = dir_components.len();
        format!("/{}", dir_components[..fallback_depth].join("/"))
    }

    fn count_paths_under(&self, prefix: &str, all_paths: &[String]) -> usize {
        let prefix_slash = format!("{}/", prefix);
        all_paths.iter().filter(|p| p.starts_with(&prefix_slash) || *p == prefix).count()
    }

    /// Freeze item
    pub fn freeze_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }

        let hashes: Vec<String> = serde_json::from_str(&params[0])?;
        let Ok(minutes) = params[1].parse::<u64>() else {
            return Err(Error::Generic(format!("Invalid minutes, {}", params[1])));
        };
        let expires_at = SystemTime::now() + Duration::from_mins(minutes);

        let mut timers = match TIMERS.write() {
            Ok(t) => t,
            Err(_) => return Err(Error::Generic("Unable to obtain lock on timers".to_string())),
        };

        // ALl files
        if hashes[0].as_str() == "all" {
            for (_, file) in self.iter_mut() {
                file.is_protected = false;
            }
            timers.push(RpcTimer {
                filename: "*".to_string(),
                expires_at,
            });
            return Ok(CmdResponse::new(true, false, message::ok(req_id, true)));
        }

        // Freeze files
        for file_hash in hashes {
            let filename = match self.iter().find(|(_k, v)| v.file_hash == file_hash) {
                Some((k, _v)) => k.clone(),
                None => return Err(Error::Validate(format!("Hash does not exist, {}", file_hash))),
            };

            if let Some(file) = self.get_mut(&filename) {
                file.is_protected = false;
                timers.push(RpcTimer {
                    filename: filename.to_string(),
                    expires_at,
                });
            }
        }

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }
}

impl BaseDbFunctions for FilesDb {
    type Item = ProtectedFile;

    #[cfg(any(target_os = "linux", feature = "fuse"))]
    /// Add new protected file
    fn add_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }

        let mut next_ino = self.values().map(|f| f.ino).max().unwrap_or(2);
        let Some(mount_dir) = get_mount_dir() else {
            return Err(Error::Generic("Unable to determine fuse mount point.".to_string()));
        };
        let mut items: Vec<ProtectedFile> = serde_json::from_str(&params[0])?;

        // Get contents of files to ensure  read permissions
        for item in items.iter_mut() {
            next_ino += 1;
            item.ino = next_ino;
            item.contents = fs::read(&item.filename)?;
        }

        // Move files and add symlinks
        for item in items {
            let symlink_path = format!("{}/{}", mount_dir, item.file_hash);
            self.insert(item.filename.to_string(), item.clone());
            fs::remove_file(&item.filename)?;
            if let Err(e) = symlink(&symlink_path, &item.filename) {
                fs::write(&item.filename, &item.contents)?;
                return Err(e.into());
            }
        }

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Delete item
    fn delete_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }
        let hashes: Vec<String> = serde_json::from_str(&params[0])?;

        // Delete
        let mut is_all = false;
        for file_hash in hashes {
            if file_hash.as_str() == "all" {
                is_all = true;
                continue;
            }

            let (filename, file) = match self.iter().find(|(_k, v)| v.file_hash == file_hash) {
                Some((k, v)) => (k.clone(), v.clone()),
                None => return Err(Error::Validate(format!("Hash does not exist, {}", file_hash))),
            };

            fs::remove_file(&file.filename)?;
            fs::write(&file.filename, &file.contents)?;
            self.remove(&filename.to_string());
        }

        if is_all {
            for (_filename, file) in self.iter() {
                fs::remove_file(&file.filename)?;
                fs::write(&file.filename, &file.contents)?;
            }
            self.clear();
        }

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Edit item
    fn edit_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Ensure item exists
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if !self.contains_key(&params[0]) {
            return Err(Error::Validate(format!("No entry to edit exists at, {}", params[0])));
        }

        // Decode JSON
        let item: Self::Item = serde_json::from_str(&params[1])?;

        // Update
        if let Some(file) = self.get_mut(&params[0]) {
            file.whitelist = item.whitelist.clone();
        }

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Get single item
    fn get_item(&self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }

        // Get item
        let mut item = match self.iter().find(|(_k, v)| v.file_hash == params[0]) {
            Some((_k, v)) => v.clone(),
            None => return Err(Error::Validate(format!("No entry exists at, {}", params[0]))),
        };
        item.contents.clear();

        Ok(CmdResponse::new(false, false, message::ok(req_id, item)))
    }

    /// List items
    fn list_items(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        let _start = if params.len() >= 2 {
            params[1].parse::<usize>().unwrap_or(0)
        } else {
            0
        };
        let filenames: Vec<String> = self.keys().map(|f| f.to_string()).collect();
        let res = self.group_files(&filenames);

        // Return
        Ok(CmdResponse::none(message::ok(req_id, res)))
    }
}

impl BaseDbItem for ProtectedFile {
    fn get_name(&self) -> String {
        self.filename.to_string()
    }
    fn set_name(&mut self, _name: &str) {
        //self.filename = name.to_string();
    }
    fn contains(&self, search: &str) -> bool {
        self.filename.to_lowercase().contains(search)
    }
}

impl Deref for FilesDb {
    type Target = HashMap<String, ProtectedFile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FilesDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Zeroize for FilesDb {
    fn zeroize(&mut self) {
        for (mut k, mut v) in self.0.drain() {
            k.zeroize();
            v.zeroize();
        }
        self.0.shrink_to_fit();
    }
}

impl Drop for FilesDb {
    fn drop(&mut self) {
        self.zeroize();
    }
}
