// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::rpc;
use falcon_cli::*;

#[derive(Default)]
pub struct CliFileList {}

impl CliCommand for CliFileList {
    fn process(&self, _req: &CliRequest) -> anyhow::Result<()> {

        // Send RPC
        let files: Vec<String> = rpc::send("file.list", &vec![&"".to_string()])?;

        // Display
        cli_header("Protected Files");
        for file in files {
        cli_sendln!("{}", file);
        }

        Ok(())
    }

    fn help(&self) -> CliHelpScreen {
        let mut help = CliHelpScreen::new(
            "List Protected Files",
            "nyx file ls] [-n XX]",
            "Lists all files currently under protection."
        );

        help.add_example("nyx file ls");
        help
    }
}
