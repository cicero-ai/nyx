// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use zeroize::Zeroize;

/// A heap-allocated buffer whose pages are mlock'd (never swapped to disk),
/// marked MADV_DONTDUMP (excluded from core dumps, Linux only), and zeroized on drop.
///
/// Use `SecureBuffer` for any transient plaintext: decrypted database bytes,
/// bincode-encoded output before encryption, intermediate key material, etc.
///
/// On Windows, only zeroize-on-drop is applied (no mlock/madvise support).
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    /// Create a new zeroed SecureBuffer of `size` bytes, locked in RAM.
    pub fn new(size: usize) -> Result<Self, Error> {
        let data = vec![0u8; size];
        let mut buf = Self { data };
        buf.protect()?;
        Ok(buf)
    }

    /// Wrap an existing Vec<u8>, locking its pages in RAM.
    /// The original Vec is consumed; its contents live in the SecureBuffer from here on.
    pub fn from_vec(data: Vec<u8>) -> Result<Self, Error> {
        let mut buf = Self { data };
        buf.protect()?;
        Ok(buf)
    }

    /// Apply platform-specific memory protections.
    fn protect(&mut self) -> Result<(), Error> {
        if self.data.is_empty() {
            return Ok(());
        }

        // Linux + macOS: mlock pages in RAM to prevent swapping to disk
        #[cfg(unix)]
        {
            let ret = unsafe {
                libc::mlock(self.data.as_ptr() as *const libc::c_void, self.data.len())
            };
            if ret != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }

        // Linux only: MADV_DONTDUMP excludes pages from core dumps.
        // macOS does not support this flag.
        #[cfg(target_os = "linux")]
        unsafe {
            libc::madvise(
                self.data.as_ptr() as *mut libc::c_void,
                self.data.len(),
                libc::MADV_DONTDUMP,
            );
        }

        // Windows: no mlock/madvise equivalent applied (zeroize-on-drop still active)

        Ok(())
    }

    /// Release mlock on the buffer's pages.
    fn unprotect(&mut self) {
        #[cfg(unix)]
        if !self.data.is_empty() {
            unsafe {
                libc::munlock(self.data.as_ptr() as *const libc::c_void, self.data.len());
            }
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consume the SecureBuffer and return the inner Vec.
    /// The caller takes responsibility for the data; mlock is released.
    pub fn into_vec(mut self) -> Vec<u8> {
        self.unprotect();
        // Take the data out so Drop doesn't zeroize it
        std::mem::take(&mut self.data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn starts_with(&self, needle: &[u8]) -> bool {
        self.data.starts_with(needle)
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // Zeroize first (while still locked in RAM), then unlock
        self.data.zeroize();
        self.unprotect();
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for SecureBuffer {
    type Output = [u8];
    fn index(&self, index: std::ops::RangeFrom<usize>) -> &[u8] {
        &self.data[index]
    }
}

impl std::ops::Index<usize> for SecureBuffer {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        &self.data[index]
    }
}
