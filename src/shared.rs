use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use std::{
    collections::HashMap,
    error::Error,
    fs::{read_to_string, write},
    path::Path,
};

const DATA_FILE_NAME: &str = "data.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ReactionRole {
    pub(crate) emoji: char,
    pub(crate) role_name: String,
}

impl ReactionRole {
    pub(crate) fn new(emoji: char, role_name: &str) -> Self {
        Self {
            emoji,
            role_name: role_name.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SetupState {
    pub(crate) channel_id: u64,
    pub(crate) guild_id: u64,
    pub(crate) post_content: Option<String>,
    pub(crate) reactions: Vec<ReactionRole>,
}

impl SetupState {
    /// Create a new, empty `SetupState`.
    pub(crate) fn new(channel_id: u64, guild_id: u64) -> Self {
        Self {
            channel_id: channel_id.to_owned(),
            guild_id: guild_id.to_owned(),
            post_content: None,
            reactions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Monitor {
    pub(crate) channel_id: u64,
    pub(crate) guild_id: u64,
    pub(crate) reactions: Vec<ReactionRole>,
}

impl Monitor {
    /// Create a new struct instance.
    pub(crate) fn new(channel_id: u64, guild_id: u64, reactions: &Vec<ReactionRole>) -> Self {
        Self {
            channel_id,
            guild_id,
            reactions: reactions.to_owned(),
        }
    }
}

pub(crate) struct MonitorManager;

impl TypeMapKey for MonitorManager {
    type Value = Vec<Monitor>;
}

impl MonitorManager {
    /// Save the manager's data to disk.
    pub(crate) fn save(&self, values: &Vec<Monitor>) -> Result<(), Box<dyn Error>> {
        let content = serde_json::to_string(values)?;
        write(DATA_FILE_NAME, content)?;
        Ok(())
    }

    /// Load the manager's data from disk.
    pub(crate) fn load() -> Result<Vec<Monitor>, Box<dyn Error>> {
        let path = Path::new(DATA_FILE_NAME);
        if !path.exists() {
            return Ok(vec![]);
        }
        let content = read_to_string(path)?;
        let parsed: Vec<Monitor> = serde_json::from_str(&content)?;
        Ok(parsed)
    }
}

pub(crate) struct StateManager;

impl TypeMapKey for StateManager {
    type Value = HashMap<String, SetupState>;
}
