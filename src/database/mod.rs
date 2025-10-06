// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use self::base::{BaseDbFunctions, BaseDbItem};
#[cfg(all(unix, feature = "fuse"))]
pub use self::fs::NyxFs;
pub use self::history::{HistoryAction, HistoryDataType, HistoryDb, HistoryItem};
pub use self::loader::LoaderResponse;
pub use self::notes::{Note, NotesDb};
pub use self::nyxdb::{DatabaseTimeout, DbStats, NyxDb};
pub use self::oauth::{Oauth, OauthDb};
pub use self::ssh_keys::{SshKey, SshKeysDb};
pub use self::strings::{StrItem, StringsDb};
pub use self::users::{User, UsersDb};

mod base;
#[cfg(all(unix, feature = "fuse"))]
mod fs;
mod history;
pub mod loader;
mod notes;
mod nyxdb;
mod oauth;
mod ssh_keys;
mod strings;
mod users;
