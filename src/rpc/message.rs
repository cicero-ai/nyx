// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use atlas_http::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: usize,
    pub method: String,
    pub params: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub id: usize,
    pub status: String,
    pub error: Option<RpcError>,
    pub result: Option<T>,
}

#[derive(Serialize, Deserialize)]
pub struct RpcError {
    pub code: usize,
    pub message: String,
}

// Generate ok response
pub fn ok<T: Serialize>(id: usize, result: T) -> HttpResponse {
    // set response
    let response = RpcResponse {
        id,
        status: "ok".to_string(),
        error: None,
        result: Some(result),
    };
    let json: String = serde_json::to_string_pretty(&response).unwrap();

    HttpResponse::new(
        &200,
        &vec!["Content-type: application/json".to_string()],
        &json,
    )
}
/// Give error response
pub fn err(id: usize, code: usize, message: &str) -> HttpResponse {
    // set response
    let response: RpcResponse<()> = RpcResponse {
        id,
        status: "error".to_string(),
        error: Some(RpcError {
            code,
            message: message.to_string(),
        }),
        result: None,
    };
    let json: String = serde_json::to_string(&response).unwrap();

    HttpResponse::new(
        &500,
        &vec!["Content-type: application/json".to_string()],
        &json,
    )
}

pub struct CmdResponse {
    pub http_res: HttpResponse,
    pub is_modified: bool,
    pub is_copy: bool,
}

impl CmdResponse {
    pub fn new(is_modified: bool, is_copy: bool, http_res: HttpResponse) -> Self {
        Self {
            http_res,
            is_modified,
            is_copy,
        }
    }

    pub fn none(http_res: HttpResponse) -> Self {
        Self::new(false, false, http_res)
    }
}
