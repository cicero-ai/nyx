// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{BaseDbFunctions, BaseDbItem};
use crate::Error;
use crate::rpc::{CmdResponse, message};
use bincode::{Decode, Encode};
use otpauth::TOTP;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

#[derive(Default, Encode, Decode)]
pub struct OauthDb(pub HashMap<String, Oauth>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Oauth {
    pub display_name: String,
    pub secret_code: String,
    pub url: String,
    pub recovery_keys: String,
}

impl OauthDb {
    /// Generate OTP code
    pub fn generate(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Get oath
        let oauth = self.get(&params[0].to_lowercase()).ok_or(Error::Validate(format!(
            "Entry does not exist at, {}",
            params[0]
        )))?;

        // Get totp client
        let client = TOTP::from_base32(&oauth.secret_code)
            .ok_or(Error::Rpc("Unable to initialize TOTP".to_string()))?;

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        // Generate OTP
        let otp_num = client.generate(30, current_time);
        let otp = format!("{:06}", otp_num);

        Ok(CmdResponse::new(false, true, message::ok(req_id, otp)))
    }
}

impl BaseDbFunctions for OauthDb {
    type Item = Oauth;

    /// Secure clear
    fn secure_clear(&mut self) {
        for (_, item) in self.iter_mut() {
            item.display_name.zeroize();
            item.secret_code.zeroize();
            item.url.zeroize();
            item.recovery_keys.zeroize();
        }
    }
}

impl BaseDbItem for Oauth {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search)
            || self.url.to_lowercase().contains(search)
    }
}

impl Deref for OauthDb {
    type Target = HashMap<String, Oauth>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OauthDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
