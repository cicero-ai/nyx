// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use self::copy::CliStrCopy;
pub use self::delete::CliStrDelete;
pub use self::find::CliStrFind;
pub use self::get::CliStrGet;
pub use self::list::CliStrList;
pub use self::rename::CliStrRename;
pub use self::set::CliStrSet;

mod copy;
mod delete;
mod find;
mod get;
mod list;
mod rename;
mod set;
