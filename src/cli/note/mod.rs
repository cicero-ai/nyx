// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use self::copy::CliNoteCopy;
pub use self::delete::CliNoteDelete;
pub use self::edit::CliNoteEdit;
pub use self::find::CliNoteFind;
pub use self::list::CliNoteList;
pub use self::new::CliNoteNew;
pub use self::rename::CliNoteRename;
pub use self::show::CliNoteShow;
pub use self::xn::CliNoteXn;

mod copy;
mod delete;
mod edit;
mod find;
mod list;
mod new;
mod rename;
mod show;
mod xn;
