// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use super::{CmdResponse, RpcRequest, message};
use crate::cli::clipboard;
use crate::database::{
    BaseDbFunctions, DatabaseTimeout, DbStats, HistoryAction, HistoryDataType, NyxDb,
};
use crate::{CONFIG, Error};
use atlas_http::HttpRequest;
use falcon_cli::*;
use std::process::exit;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::task;

#[cfg(unix)]
use fuser::BackgroundSession;

pub struct RpcDaemon {
    pub nyxdb: Arc<Mutex<NyxDb>>,
    pub session: Mutex<RpcSession>,
    pub fuse_point: Mutex<Option<BackgroundSession>>,
}

pub struct RpcSession {
    pub dbfile: String,
    pub lock: [u8; 32],
    pub is_modified: bool,
    pub timeout: DatabaseTimeout,
    pub clipboard_timeout: u64,
    pub expires_at: Option<SystemTime>,
    pub clipboard_expires_at: Option<SystemTime>,
}

impl RpcDaemon {
    pub fn new(nyxdb: NyxDb, dbfile: &str, n_password: [u8; 32]) -> Self {
        Self {
            session: Mutex::new(RpcSession::new(&nyxdb, dbfile, n_password)),
            nyxdb: Arc::new(Mutex::new(nyxdb)),
            fuse_point: Mutex::new(None),
        }
    }

    /// Start the daemon
    pub async fn start(self: Arc<Self>) -> Result<(), Error> {
        #[cfg(unix)]
        // Mount fuse point
        {
            if let Err(e) = super::fs_launcher::mount(&self) {
                cli_error!("Unable to mouint fuse point, skipping.  Error: {}", e);
            }
        }

        // Bind to localhost
        let rpc_host = format!("{}:{}", CONFIG.host, CONFIG.port);
        let listener = TcpListener::bind(&rpc_host).await?;
        cli_info!("Listening for connections on {}...", rpc_host);

        // Create timer
        let mut timer_interval = tokio::time::interval(tokio::time::Duration::from_secs(15));

        // Listen for connections
        loop {
            tokio::select! {
                _ = timer_interval.tick() => {
                    self.check_timer().await;
                },
                accept_result = listener.accept() => {

                    let (mut stream, _) = match accept_result {
                        Ok(r) => r,
                        Err(_) => continue
                    };

                    let handler_self = Arc::clone(&self);
                    task::spawn(async move {
                        if let Ok(req) = HttpRequest::build_async(&mut stream).await {

                            // Handle request
                            let res = handler_self.handle(req);

                            // Output response
                            stream.write_all(res.http_res.raw().as_bytes()).await.unwrap();
                        }
                    });

                }
            };
        }
    }

    /// Handle incoming connection
    fn handle(&self, http_req: HttpRequest) -> CmdResponse {
        // Get HTTP body contents
        let json_str = match String::from_utf8(http_req.body.get_raw()) {
            Ok(r) => r,
            Err(_) => return CmdResponse::none(message::err(0, 400, "Invalid JSON")),
        };

        // Decode JSON
        let req: RpcRequest = match serde_json::from_str(&json_str) {
            Ok(r) => r,
            Err(_) => return CmdResponse::none(message::err(0, 500, "Invalid JSON request")),
        };

        // Check for valid method
        let parts: Vec<String> = req.method.split(".").map(|w| w.to_string()).collect();
        if parts.len() != 2 {
            return CmdResponse::none(message::err(
                req.id,
                404,
                &format!("Method does not exist, {}", req.method),
            ));
        }

        // Shutdown
        if req.method.as_str() == "db.close" {
            self.shutdown();
        }

        // Lock data store
        let mut db = match self.nyxdb.lock() {
            Ok(r) => r,
            Err(e) => {
                return CmdResponse::none(message::err(
                    req.id,
                    500,
                    &format!("Unable to lock database, {}", e),
                ));
            }
        };

        // Route request
        let wrapped_res = match (parts[0].as_str(), parts[1].as_str()) {
            // Database
            ("db", "history") => db.history.list_items(req.id, &req.params),
            ("db", "stats") => self.dbstats(req.id, &mut db),

            // Users
            ("user", "copy") => db.users.copy_item(req.id, &req.params),
            ("user", "delete") => db.users.delete_item(req.id, &req.params),
            ("user", "edit") => db.users.edit_item(req.id, &req.params),
            ("user", "exists") => db.users.exists(req.id, &req.params),
            ("user", "find") => db.users.find_items(req.id, &req.params),
            ("user", "get") => db.users.get_item(req.id, &req.params),
            ("user", "list") => db.users.list_items(req.id, &req.params),
            ("user", "new") => db.users.add_item(req.id, &req.params),
            ("user", "rename") => db.users.rename_item(req.id, &req.params),

            // Oaut / OTP
            ("otp", "copy") => db.oauth.copy_item(req.id, &req.params),
            ("otp", "delete") => db.oauth.delete_item(req.id, &req.params),
            ("otp", "edit") => db.oauth.edit_item(req.id, &req.params),
            ("otp", "exists") => db.oauth.exists(req.id, &req.params),
            ("otp", "find") => db.oauth.find_items(req.id, &req.params),
            ("otp", "generate") => db.oauth.generate(req.id, &req.params),
            ("otp", "get") => db.oauth.get_item(req.id, &req.params),
            ("otp", "list") => db.oauth.list_items(req.id, &req.params),
            ("otp", "new") => db.oauth.add_item(req.id, &req.params),
            ("otp", "rename") => db.oauth.rename_item(req.id, &req.params),

            // SSH keys
            ("ssh", "copy") => db.ssh_keys.copy_key(req.id, &req.params),
            ("ssh", "delete") => db.ssh_keys.delete_key(req.id, &req.params),
            ("ssh", "edit") => db.ssh_keys.edit_item(req.id, &req.params),
            ("ssh", "exists") => db.ssh_keys.exists(req.id, &req.params),
            ("ssh", "find") => db.ssh_keys.find_items(req.id, &req.params),
            ("ssh", "generate") => db.ssh_keys.generate(req.id, &req.params),
            ("ssh", "import") => db.ssh_keys.import(req.id, &req.params),
            ("ssh", "get") => db.ssh_keys.get_item(req.id, &req.params),
            ("ssh", "list") => db.ssh_keys.list_items(req.id, &req.params),
            ("ssh", "rename") => db.ssh_keys.rename_key(req.id, &req.params),

            // Strings
            ("str", "copy") => db.strings.copy_item(req.id, &req.params),
            ("str", "delete") => db.strings.delete_item(req.id, &req.params),
            ("str", "exists") => db.strings.exists(req.id, &req.params),
            ("str", "find") => db.strings.find_items(req.id, &req.params),
            ("str", "get") => db.strings.get_item(req.id, &req.params),
            ("str", "list") => db.strings.list_items(req.id, &req.params),
            ("str", "rename") => db.strings.rename_item(req.id, &req.params),
            ("str", "set") => db.strings.add_item(req.id, &req.params),

            // Notes
            ("note", "copy") => db.notes.copy_item(req.id, &req.params),
            ("note", "delete") => db.notes.delete_item(req.id, &req.params),
            ("note", "edit") => db.notes.edit_item(req.id, &req.params),
            ("note", "exists") => db.notes.exists(req.id, &req.params),
            ("note", "find") => db.notes.find_items(req.id, &req.params),
            ("note", "get") => db.notes.get_item(req.id, &req.params),
            ("note", "list") => db.notes.list_items(req.id, &req.params),
            ("note", "new") => db.notes.add_item(req.id, &req.params),
            ("note", "rename") => db.notes.rename_item(req.id, &req.params),

            _ => Ok(CmdResponse::none(message::err(
                0,
                404,
                &format!("Method does not exist, {}", req.method),
            ))),
        };

        // Check response
        let res = match wrapped_res {
            Ok(r) => r,
            Err(e) => return CmdResponse::none(message::err(req.id, 500, &e.to_string())),
        };

        // Add history
        if let Ok(history_action) = HistoryAction::from_str(&parts[1])
            && let Ok(data_type) = HistoryDataType::from_str(&parts[0])
        {
            let dest = if ["copy", "rename"].contains(&parts[1].as_str()) {
                req.params[1].to_string()
            } else {
                "".to_string()
            };
            if let Err(e) = db.history.add(history_action, data_type, &req.params[0], &dest) {
                return CmdResponse::none(message::err(
                    req.id,
                    500,
                    &format!("Unable to add history entry: {}", e),
                ));
            }

            if let Err(e) = self.savedb(req.id, &mut db) {
                return CmdResponse::none(message::err(
                    req.id,
                    500,
                    &format!("Unable to save database: {}", e),
                ));
            }
        }

        // Update session
        self.update_session(&res);

        res
    }

    fn update_session(&self, res: &CmdResponse) {
        // Lock session
        let mut session = match self.session.lock() {
            Ok(r) => r,
            Err(_) => return,
        };

        // Get expires at
        session.expires_at = if let DatabaseTimeout::Duration(duration) = session.timeout {
            Some(SystemTime::now() + duration)
        } else {
            None
        };

        if res.is_modified {
            session.is_modified = true;
        }

        if res.is_copy {
            session.clipboard_expires_at =
                Some(SystemTime::now() + Duration::from_secs(session.clipboard_timeout));
        }
    }

    /// Save database
    fn savedb(&self, req_id: usize, db: &mut NyxDb) -> Result<CmdResponse, Error> {
        // Lock session
        let mut session =
            self.session.lock().map_err(|e| Error::Db(format!("Unable to load session: {}", e)))?;

        // Save
        db.save(&session.dbfile, session.lock, None)?;
        session.is_modified = false;

        Ok(CmdResponse::none(message::ok(req_id, true)))
    }

    /// Get database stats
    fn dbstats(&self, req_id: usize, db: &mut NyxDb) -> Result<CmdResponse, Error> {
        // Lock session
        let session = match self.session.lock() {
            Ok(r) => r,
            Err(e) => return Err(Error::Db(format!("Unable to lock session: {}", e))),
        };

        let stats = DbStats::new(&session.dbfile, db);
        Ok(CmdResponse::none(message::ok(req_id, stats)))
    }

    /// Shutdown
    fn shutdown(&self) {
        // Secure clear database
        if let Ok(mut db) = self.nyxdb.lock() {
            db.secure_clear();
        }

        cli_info!("Received shutdown order, gracefully exiting.\n");
        exit(0);
    }

    /// Check timer
    async fn check_timer(&self) {
        // Lock session
        let mut session = match self.session.lock() {
            Ok(r) => r,
            Err(_) => return,
        };

        // Check clearing of clipboard
        if let Some(clip_expires_at) = session.clipboard_expires_at
            && SystemTime::now() >= clip_expires_at
        {
            let _ = clipboard::copy("");
            session.clipboard_expires_at = None;
        }

        // Check database expiration
        if let Some(expires_at) = session.expires_at
            && SystemTime::now() > expires_at
        {
            self.shutdown();
        }
    }
}

impl RpcSession {
    pub fn new(nyxdb: &NyxDb, dbfile: &str, lock: [u8; 32]) -> Self {
        let timeout = if let Some(to) = CONFIG.timeout {
            to
        } else {
            nyxdb.default_timeout
        };

        // Get expires at
        let expires_at = if let DatabaseTimeout::Duration(duration) = timeout {
            Some(SystemTime::now() + duration)
        } else {
            None
        };

        Self {
            dbfile: dbfile.to_string(),
            lock,
            timeout,
            clipboard_timeout: CONFIG.clipboard_timeout,
            is_modified: false,
            expires_at,
            clipboard_expires_at: None,
        }
    }
}
