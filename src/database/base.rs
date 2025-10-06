// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use crate::rpc::{CmdResponse, message};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

pub trait BaseDbItem {
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: &str);
    fn contains(&self, search: &str) -> bool;
}

pub trait BaseDbFunctions:
    Deref<Target = HashMap<String, Self::Item>> + DerefMut<Target = HashMap<String, Self::Item>>
where
    Self::Item: Clone + Serialize + for<'a> Deserialize<'a> + BaseDbItem,
{
    type Item;

    fn secure_clear(&mut self);

    /// Add new item
    fn add_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }

        let item: Self::Item = serde_json::from_str(&params[1])?;

        // Check if exists
        if self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!(
                "Entry already exists, {}",
                params[0]
            )));
        }

        // Insert
        self.insert(params[0].to_lowercase(), item);
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Copy item
    fn copy_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.len() < 2 {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if self.contains_key(&params[1].to_lowercase()) {
            return Err(Error::Validate(format!(
                "Destination to copy item to already exists, {}",
                params[1]
            )));
        }

        // Get item
        let item = self.get(&params[0].to_lowercase()).ok_or(Error::Validate(format!(
            "Entry to copy  does not exist at, {}",
            params[0]
        )))?;

        // Copy
        let mut new_item = item.clone();
        new_item.set_name(&params[1]);

        // Insert
        self.insert(params[1].to_lowercase(), new_item);
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Delete item
    fn delete_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if !self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!(
                "No entry to delete exists at {}",
                params[0]
            )));
        }

        // Delete
        self.remove(&params[0].to_lowercase());
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Edit item
    fn edit_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Ensure item exists
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if !self.contains_key(&params[0].to_lowercase()) {
            return Err(Error::Validate(format!(
                "No entry to edit exists at, {}",
                params[0]
            )));
        }

        // Decode JSON
        let item: Self::Item = serde_json::from_str(&params[1])?;

        // Update
        self.insert(params[0].to_lowercase(), item);
        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }

    /// Check whether or not item exists
    fn exists(&self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }

        if self.contains_key(&params[0].to_lowercase()) {
            Ok(CmdResponse::none(message::ok(req_id, true)))
        } else {
            Ok(CmdResponse::none(message::ok(req_id, false)))
        }
    }

    /// Find items
    fn find_items(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }
        let search = params[0].to_lowercase();

        // Get items
        let mut items: Vec<String> = self
            .values()
            .filter(|&item| item.contains(&search))
            .map(|item| item.get_name())
            .collect();

        // Sort and reply
        items.sort();
        Ok(CmdResponse::none(message::ok(req_id, items)))
    }

    /// Get single item
    fn get_item(&self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        if params.is_empty() {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        }

        // Get item
        let item = self.get(&params[0].to_lowercase()).ok_or(Error::Validate(format!(
            "No entry exists at, {}",
            params[0]
        )))?;

        // Check if password copied
        let is_copy = params.len() > 1 && params[1].as_str() == "1";
        Ok(CmdResponse::new(false, is_copy, message::ok(req_id, item)))
    }

    /// List items
    fn list_items(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Get dirname
        let dirname = if params[0].is_empty() {
            String::new()
        } else {
            format!("{}/", params[0])
        };
        let (mut dirs, mut files) = (HashSet::new(), vec![]);

        let start = if params.len() >= 2 {
            params[1].parse::<usize>().unwrap_or(0)
        } else {
            0
        };

        // Get items
        for value in self.keys() {
            if !value.starts_with(&dirname) {
                continue;
            }
            let name_str = value.trim_start_matches(&dirname).to_string();

            if name_str.contains("/") {
                if let Some(short_name) = name_str.split("/").next() {
                    dirs.insert(format!("{}/", short_name));
                }
            } else {
                files.push(name_str);
            }
        }

        // Sort and finish items
        let mut items: Vec<String> = dirs.into_iter().collect();
        items.sort();
        files.sort();
        items.extend(files);

        // Get items to display
        let end = (start + 25).min(items.len());
        let res = if start >= items.len() {
            vec![]
        } else {
            items[start..end].to_vec()
        };

        // Return
        Ok(CmdResponse::none(message::ok(req_id, res)))
    }

    /// Rename item
    fn rename_item(&mut self, req_id: usize, params: &Vec<String>) -> Result<CmdResponse, Error> {
        // Validate
        if params.len() < 2 {
            return Err(Error::Validate("Invalid parameters.".to_string()));
        } else if self.contains_key(&params[1].to_lowercase()) {
            return Err(Error::Validate(format!(
                "Destination to rename item to already exists, {}",
                params[1]
            )));
        }

        // Get item
        let item = self.get(&params[0].to_lowercase()).ok_or(Error::Validate(format!(
            "No entry exists at, {}",
            params[0]
        )))?;

        // Copy
        let mut new_item = item.clone();
        new_item.set_name(&params[1]);

        // Insert
        self.insert(params[1].to_lowercase(), new_item);
        self.remove(&params[0].to_lowercase());

        Ok(CmdResponse::new(true, false, message::ok(req_id, true)))
    }
}
