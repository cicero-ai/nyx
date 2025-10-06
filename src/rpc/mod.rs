// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use self::daemon::RpcDaemon;
pub use self::message::{CmdResponse, RpcRequest, RpcResponse};
use crate::database::loader;
use crate::{CONFIG, Error};
use atlas_http::{HttpBody, HttpClient, HttpRequest};
use falcon_cli::*;
use rand::Rng;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::process::exit;

mod daemon;
pub mod launcher;
pub mod message;

#[cfg(all(unix, feature = "fuse"))]
pub mod fs_launcher;

/// Send request
pub fn send<T, R>(method: &str, params: &Vec<T>) -> Result<R, Error>
where
    T: Serialize + Display + Send + Sync,
    R: DeserializeOwned + 'static,
{
    let mut rng = rand::thread_rng();

    // Ping, and check if RPC server online
    if !launcher::ping() {
        let (dbfile, n_password) = match loader::load() {
            Ok(r) => r,
            Err(e) => {
                cli_error!("Unable to load Nyx database: {}", e);
                exit(1);
            }
        };

        if let Err(e) = launcher::launch(&dbfile, n_password) {
            cli_error!("Unable to launch RPC daemon: {}", e);
            exit(1);
        }
    }

    // Create json request
    let req = RpcRequest {
        id: rng.gen_range(100000..1000000),
        method: method.to_string(),
        params: params.iter().map(|p| p.to_string()).collect(),
    };
    let json_str = serde_json::to_string(&req).unwrap();

    // Create http request
    let url = format!("http://{}:{}/", CONFIG.host, CONFIG.port);
    let req = HttpRequest::new(
        "POST",
        &url,
        &vec!["Content-type: application/json"],
        &HttpBody::from_raw(json_str.as_bytes()),
    );

    // Send http request
    let mut http = HttpClient::builder().build_sync();
    let http_res = http.send(&req)?;

    // Decode json
    let json_res: RpcResponse<R> = serde_json::from_str(&http_res.body())?;

    // Check status
    if let Some(error) = json_res.error {
        let errmsg = format!("#{} - {}", error.code, error.message);
        return Err(Error::Rpc(errmsg));
    }

    if let Some(res) = json_res.result {
        Ok(res)
    } else {
        Err(Error::Rpc("Received an empty response.".to_string()))
    }
}
