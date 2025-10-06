// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{BaseDbFunctions, BaseDbItem};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use zeroize::Zeroize;

#[derive(Default, Encode, Decode)]
pub struct UsersDb(pub HashMap<String, User>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize)]
pub struct User {
    pub display_name: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

impl BaseDbFunctions for UsersDb {
    type Item = User;

    /// Secure clear
    fn secure_clear(&mut self) {
        for (_, user) in self.iter_mut() {
            user.display_name.zeroize();
            user.username.zeroize();
            user.password.zeroize();
            user.url.zeroize();
            user.notes.zeroize();
        }
    }
}

impl BaseDbItem for User {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search)
            || self.username.to_lowercase().contains(search)
            || self.url.to_lowercase().contains(search)
    }
}

impl Deref for UsersDb {
    type Target = HashMap<String, User>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UsersDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
