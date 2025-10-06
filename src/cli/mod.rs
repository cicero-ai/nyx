// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc;
use falcon_cli::*;

use self::db::{
    CliDbBackup, CliDbChangePass, CliDbClose, CliDbCreate, CliDbHistory, CliDbOpen, CliDbRestore,
    CliDbStats,
};
use self::note::{
    CliNoteCopy, CliNoteDelete, CliNoteEdit, CliNoteFind, CliNoteList, CliNoteNew, CliNoteRename,
    CliNoteShow, CliNoteXn,
};
use self::otp::{
    CliOtpCopy, CliOtpDelete, CliOtpEdit, CliOtpFind, CliOtpGenerate, CliOtpList, CliOtpNew,
    CliOtpRename, CliOtpShow, CliOtpXp, CliOtpXr, CliOtpXw,
};
use self::ssh::{
    CliSshKeyCopy, CliSshKeyDelete, CliSshKeyEdit, CliSshKeyFind, CliSshKeyGenerate,
    CliSshKeyImport, CliSshKeyList, CliSshKeyRename, CliSshKeyShow, CliSshKeyXb, CliSshKeyXh,
    CliSshKeyXp, CliSshKeyXu, CliSshKeyXv,
};
use self::str::{
    CliStrCopy, CliStrDelete, CliStrFind, CliStrGet, CliStrList, CliStrRename, CliStrSet,
};
use self::user::{
    CliUserCopy, CliUserDelete, CliUserEdit, CliUserFind, CliUserList, CliUserNew, CliUserRename,
    CliUserShow, CliUserXp, CliUserXu, CliUserXw,
};

#[cfg(feature="testutil")]
use self::test::CliTest;

pub mod clipboard;
mod db;
mod note;
mod otp;
mod ssh;
mod str;
mod user;

#[cfg(feature="testutil")]
mod test;

/// Boot CLI router and define available commands
pub fn boot() -> CliRouter {
    let mut router = CliRouter::new();
    router.app_name("Nyx");
    router.version_message(&format!("Nyx v{} - Secure CLI password & key manager\nDeveloped by the Cicero Project - https://cicero.sh/latest", env!("CARGO_PKG_VERSION")));

    router.global("-f", "--dbfile", true, "Location of Nyx database file.");
    router.global("-t", "--timeout", true, "Time of inactivity to lock database (eg. 3h = 3 hours, 15m = 15 minutes, 60s = 60 seconds)");
    router.global(
        "-c",
        "--cb-timeout",
        true,
        "Seconds to auto-clear clipboard, default 120.",
    );
    router.global(
        "-m",
        "--mount-dir",
        true,
        "Directory to mount fuse point, defaults to /tmp/nyx",
    );
    router.global("-h", "--host", true, "RPC host, defaults to 127.0.0.1");
    router.global("-p", "--port", true, "RPC port, defaults to 7924");
    router.ignore("-d", false);

    // db
    router.add_category("db", "Database", "Manage Nyx database files.");
    router.add::<CliDbBackup>("db backup", vec!["backup"], vec![]);
    router.add::<CliDbChangePass>("db changepass", vec!["changepass"], vec![]);
    router.add::<CliDbClose>("db close", vec!["close"], vec![]);
    router.add::<CliDbCreate>("db create", vec![], vec![]);
    router.add::<CliDbHistory>("db history", vec!["history"], vec![]);
    router.add::<CliDbOpen>("db open", vec!["open"], vec![]);
    router.add::<CliDbRestore>("db restore", vec!["restore"], vec![]);
    router.add::<CliDbStats>("db stats", vec!["stats"], vec![]);

    // Users
    router.add_category("user", "Users", "Manage user / password combinations.");
    router.add::<CliUserCopy>("user cp", vec!["user copy", "copy", "cp"], vec![]);
    router.add::<CliUserDelete>("user rm", vec!["user del", "rm", "del"], vec![]);
    router.add::<CliUserEdit>("user edit", vec!["edit"], vec![]);
    router.add::<CliUserFind>("user find", vec!["find"], vec![]);
    router.add::<CliUserList>("user ls", vec!["user list", "list", "ls"], vec!["-n"]);
    router.add::<CliUserNew>("user new", vec!["new"], vec![]);
    router.add::<CliUserRename>("user mv", vec!["user rename", "rename", "mv"], vec![]);
    router.add::<CliUserShow>("user show", vec!["show"], vec![]);
    router.add::<CliUserXp>("user xp", vec!["xp"], vec![]);
    router.add::<CliUserXu>("user xu", vec!["xu"], vec![]);
    router.add::<CliUserXw>("user xw", vec!["xw"], vec![]);

    // Oauth OTP codes
    router.add_category("otp", "Oauth", "Manage Oauth OTP Codes");
    router.add::<CliOtpGenerate>("otp", vec![], vec![]);
    router.add::<CliOtpCopy>("otp cp", vec!["otp copy"], vec![]);
    router.add::<CliOtpDelete>("otp rm", vec!["otp delete", "otp del"], vec![]);
    router.add::<CliOtpEdit>("otp edit", vec![], vec![]);
    router.add::<CliOtpFind>("otp find", vec![], vec![]);
    router.add::<CliOtpList>("otp ls", vec!["opt list"], vec!["-n"]);
    router.add::<CliOtpNew>("otp new", vec![], vec![]);
    router.add::<CliOtpRename>("otp mv", vec!["otp rename"], vec![]);
    router.add::<CliOtpShow>("otp show", vec![], vec![]);
    router.add::<CliOtpXp>("otp xp", vec![], vec![]);
    router.add::<CliOtpXr>("otp xr", vec![], vec![]);
    router.add::<CliOtpXw>("otp xw", vec![], vec![]);

    // SSH keys
    router.add_category("ssh", "SSH Keys", "Manage SSH keys");
    router.add::<CliSshKeyCopy>("ssh cp", vec!["ssh copy"], vec![]);
    router.add::<CliSshKeyDelete>("ssh rm", vec!["ssh delete", "ssh del"], vec![]);
    router.add::<CliSshKeyEdit>("ssh edit", vec![], vec![]);
    router.add::<CliSshKeyFind>("ssh find", vec![], vec![]);
    router.add::<CliSshKeyGenerate>("ssh gen", vec!["ssh generate"], vec![]);
    router.add::<CliSshKeyImport>("ssh import", vec![], vec!["--file"]);
    router.add::<CliSshKeyList>("ssh ls", vec!["ssh list"], vec!["-n"]);
    router.add::<CliSshKeyRename>("ssh mv", vec!["ssh rename"], vec![]);
    router.add::<CliSshKeyShow>("ssh show", vec![], vec![]);
    router.add::<CliSshKeyXb>("ssh xb", vec![], vec![]);
    router.add::<CliSshKeyXh>("ssh xh", vec![], vec![]);
    router.add::<CliSshKeyXp>("ssh xp", vec![], vec![]);
    router.add::<CliSshKeyXu>("ssh xu", vec![], vec![]);
    router.add::<CliSshKeyXv>("ssh xv", vec![], vec![]);

    // Strings
    router.add_category("str", "Strings", "Manage strings");
    router.add::<CliStrCopy>("str cp", vec!["str copy"], vec![]);
    router.add::<CliStrDelete>("str rm", vec!["str delete", "str del"], vec![]);
    router.add::<CliStrFind>("str find", vec![], vec![]);
    router.add::<CliStrGet>("str get", vec!["get"], vec![]);
    router.add::<CliStrList>("str ls", vec!["str list"], vec!["-n"]);
    router.add::<CliStrRename>("str mv", vec!["str rename"], vec![]);
    router.add::<CliStrSet>("str set", vec!["set"], vec![]);

    // Notes
    router.add_category("note", "Notes", "Manage notes / text files.");
    router.add::<CliNoteCopy>("note cp", vec!["note copy"], vec![]);
    router.add::<CliNoteDelete>("note rm", vec!["note delete", "note del"], vec![]);
    router.add::<CliNoteEdit>("note edit", vec![], vec![]);
    router.add::<CliNoteFind>("note find", vec![], vec![]);
    router.add::<CliNoteList>("note ls", vec!["note list"], vec!["-n"]);
    router.add::<CliNoteNew>("note new", vec![], vec![]);
    router.add::<CliNoteRename>("note mv", vec!["note rename"], vec![]);
    router.add::<CliNoteShow>("note show", vec![], vec![]);
    router.add::<CliNoteXn>("note xn", vec![], vec![]);

    // Test utils
    #[cfg(feature="testutil")]
    {
        router.add::<CliTest>("test", vec![], vec![]);
    }

    router
}

/// Perform existence check on an item
pub fn check_exists(category: &str, item_name: &str, expected_bool: bool) -> Result<(), CliError> {
    let method_name = format!("{}.exists", category);

    let exists: bool =
        rpc::send(&method_name, &vec![item_name]).map_err(|e| CliError::Generic(e.to_string()))?;

    if exists && !expected_bool {
        return Err(CliError::Generic(format!(
            "Entry with that name already exists, {}",
            item_name
        )));
    } else if (!exists) && expected_bool {
        return Err(CliError::Generic(format!(
            "No entry exists with the name, {}",
            item_name
        )));
    }

    Ok(())
}
