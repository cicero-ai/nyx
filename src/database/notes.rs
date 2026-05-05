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

#[derive(Default, Encode, Decode)]
pub struct NotesDb(pub HashMap<String, Note>);

#[derive(Clone, Encode, Decode, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct Note {
    pub display_name: String,
    pub note: String,
}

impl BaseDbFunctions for NotesDb {
    type Item = Note;
}

impl BaseDbItem for Note {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    fn set_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    fn contains(&self, search: &str) -> bool {
        self.display_name.to_lowercase().contains(search) || self.note.to_lowercase().contains(search)
    }
}

impl Deref for NotesDb {
    type Target = HashMap<String, Note>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NotesDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Zeroize for NotesDb {
    fn zeroize(&mut self) {
        for (mut k, mut v) in self.0.drain() {
            k.zeroize();
            v.zeroize();
        }
        self.0.shrink_to_fit();
    }
}

impl Drop for NotesDb {
    fn drop(&mut self) {
        self.zeroize();
    }
}
