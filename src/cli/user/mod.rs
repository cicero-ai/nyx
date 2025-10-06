// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub use self::copy::CliUserCopy;
pub use self::delete::CliUserDelete;
pub use self::edit::CliUserEdit;
pub use self::find::CliUserFind;
pub use self::list::CliUserList;
pub use self::new::CliUserNew;
pub use self::rename::CliUserRename;
pub use self::show::CliUserShow;
pub use self::xp::CliUserXp;
pub use self::xu::CliUserXu;
pub use self::xw::CliUserXw;

mod copy;
mod delete;
mod edit;
mod find;
mod list;
mod new;
mod rename;
mod show;
mod xp;
mod xu;
mod xw;
