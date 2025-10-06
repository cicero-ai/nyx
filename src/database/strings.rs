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
pub struct StringsDb(pub HashMap<String, StrItem>);

#[derive(Clone, Decode, Encode, Serialize, Deserialize)]
pub struct StrItem {
    pub display_name: String,
    pub value: String,
}

impl BaseDbFunctions for StringsDb {
    type Item = StrItem;

    /// Secure clear
    fn secure_clear(&mut self) {
        for (_, item) in self.iter_mut() {
            item.display_name.zeroize();
            item.value.zeroize();
        }
    }
}

impl BaseDbItem for StrItem {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search)
            || self.value.to_lowercase().contains(search)
    }
}

impl Deref for StringsDb {
    type Target = HashMap<String, StrItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StringsDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
