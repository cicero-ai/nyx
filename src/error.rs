// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use falcon_cli::anyhow;
use std::fmt;
use std::time::SystemTimeError;
#[derive(Debug)]
pub enum Error {
    Db(String),
    Io(String),
    Crypto(String),
    Http(String),
    Json(String),
    Rpc(String),
    Validate(String),
    Generic(String),
    InvalidArguments,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Db(err) => write!(f, "Database error: {}", err),
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::Crypto(err) => write!(f, "Crypto error: {}", err),
            Error::Http(err) => write!(f, "HTTP error: {}", err),
            Error::Json(err) => write!(f, "JSON error: {}", err),
            Error::Rpc(err) => write!(f, "RPC error: {}", err),
            Error::Validate(err) => write!(f, "{}", err),
            Error::Generic(msg) => write!(f, "{}", msg),
            Error::InvalidArguments => write!(f, "Invalid arguments"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<atlas_http::error::Error> for Error {
    fn from(err: atlas_http::error::Error) -> Self {
        Error::Http(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Error::Generic(e.to_string())
    }
}

impl From<falcon_cli::CliError> for Error {
    fn from(e: falcon_cli::CliError) -> Self {
        Error::Validate(e.to_string())
    }
}

impl From<crate::Error> for falcon_cli::CliError {
    fn from(e: crate::Error) -> Self {
        falcon_cli::CliError::Generic(e.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Generic(err.to_string())
    }
}
