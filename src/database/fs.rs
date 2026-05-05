// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{FilesDb, NyxDb};
use fuser::{
    FileType, Filesystem, KernelConfig, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyOpen, ReplyWrite, Request,
};
use libc::{EACCES, ENOENT};
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
        self.0.lock().unwrap().files.lookup(_req, parent, name, reply)
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply_ino: Option<u64>, reply: ReplyAttr) {
        self.0.lock().unwrap().files.getattr(_req, ino, reply_ino, reply)
    }

    fn open(&mut self, _req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        self.0.lock().unwrap().files.open(_req, ino, _flags, reply)
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
        self.0.lock().unwrap().files.read(_req, ino, _fh, offset, size, flags, _lock_owner, reply)
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, reply: ReplyDirectory) {
        self.0.lock().unwrap().files.readdir(_req, ino, _fh, offset, reply)
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        self.0.lock().unwrap().files.write(_req, ino, fh, offset, data, write_flags, flags, lock_owner, reply)
    }

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        crtime: Option<std::time::SystemTime>,
        chgtime: Option<std::time::SystemTime>,
        bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        self.0.lock().unwrap().files.setattr(
            _req, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime, flags, reply,
        )
    }
}

impl Filesystem for FilesDb {
    fn init(&mut self, _req: &Request, _config: &mut KernelConfig) -> Result<(), libc::c_int> {
        Ok(())
    }

    // Lookup directory by name
    fn lookup(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEntry) {
        // Get directory name
        let name_str = match name.to_str() {
            Some(r) => r.trim_start_matches("/").to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        // Check for root /.mount_status/
        if name_str == "." {
            let attr = self.get_attr(1).unwrap();
            reply.entry(&TTL, &attr, 0);
            return;
        } else if name_str == ".mount_status" {
            let attr = self.get_attr(2).unwrap();
            reply.entry(&TTL, &attr, 0);
            return;
        }
        //name_str = name_str.trim_start_matches("/").to_string();

        // Lookup info of file
        let ino = match self.values().find(|f| *OsStr::new(f.file_hash.as_str()) == *name) {
            Some(v) => v.ino,
            None => {
                reply.error(ENOENT);
                return;
            }
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
    fn open(&mut self, req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        if ino == 2 {
            reply.error(EACCES);
            return;
        }

        if let Some(file) = self.values().find(|f| f.ino == ino) {
            if self.is_allowed(req, file) {
                reply.opened(0, 0);
            } else {
                reply.error(EACCES);
            }
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
        if let Some(prfile) = self.values().find(|f| f.ino == ino) {
            let start = offset as usize;
            let end = (offset as usize + size as usize).min(prfile.contents.len());

            if start < prfile.contents.len() {
                reply.data(&prfile.contents[start..end]);
            } else {
                reply.data(&[]);
            }
            return;
        }

        reply.error(ENOENT);
    }

    /// Read directory  entries
    fn readdir(&mut self, _req: &Request, _ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        // Root entries
        let mut entries = vec![
            (INO_ROOT, FileType::Directory, Path::new(".")),
            (INO_ROOT, FileType::Directory, Path::new("..")),
            (2, FileType::RegularFile, Path::new(".mount_status")),
        ];

        // Add directories
        for (_, file) in self.iter() {
            entries.push((file.ino, FileType::RegularFile, Path::new(&file.file_hash)));
        }

        for (i, (ino, kind, name)) in entries.into_iter().enumerate().skip(offset as usize) {
            let next_offset = (i + 1) as i64;
            if reply.add(ino, next_offset, kind, name) {
                break;
            }
        }

        reply.ok();
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        let filename = match self.values().find(|f| f.ino == ino) {
            Some(r) => r.filename.to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        if let Some(file) = self.get_mut(&filename) {
            let start = offset as usize;
            let end = start + data.len();
            if file.contents.len() < end {
                file.contents.resize(end, 0);
            }
            file.contents[start..end].copy_from_slice(data);
            reply.written(data.len() as u32);
        } else {
            reply.error(ENOENT);
        }
    }

    fn create(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        reply.error(EACCES);
    }

    fn unlink(&mut self, _req: &Request<'_>, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        reply.error(EACCES);
    }

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let filename = match self.values().find(|f| f.ino == ino) {
            Some(r) => r.filename.to_string(),
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        if let Some(file) = self.get_mut(&filename) {
            if let Some(new_size) = size {
                file.contents.resize(new_size as usize, 0);
            }
            // Return updated attrs
            if let Some(attr) = self.get_attr(ino) {
                reply.attr(&TTL, &attr);
            } else {
                reply.error(ENOENT);
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: fuser::ReplyEmpty) {
        reply.ok();
    }

    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: fuser::ReplyEmpty) {
        reply.ok();
    }
}
