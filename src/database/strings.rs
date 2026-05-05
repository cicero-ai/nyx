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
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Default, Encode, Decode, Serialize, Deserialize)]
pub struct StringsDb(pub HashMap<String, StrItem>);

#[derive(Clone, Decode, Encode, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct StrItem {
    pub display_name: String,
    pub value: String,
}

impl BaseDbFunctions for StringsDb {
    type Item = StrItem;
}

impl BaseDbItem for StrItem {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search) || self.value.to_lowercase().contains(search)
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

impl Zeroize for StringsDb {
    fn zeroize(&mut self) {
        for (mut k, mut v) in self.0.drain() {
            k.zeroize();
            v.zeroize();
        }
        self.0.shrink_to_fit();
    }
}

impl Drop for StringsDb {
    fn drop(&mut self) {
        self.zeroize();
    }
}
