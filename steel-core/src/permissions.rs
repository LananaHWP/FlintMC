//! Permission system for operators and access control.
//!
//! This module provides operator functionality similar to vanilla's ops.json.
//! It supports permission levels 0-4, where:
//! - Level 0: Non-operator (default for all players)
//! - Level 1: Operator (can bypass spawn protection)
//! - Level 2: Operator (can use commands like /give, /summon, /tm, /effect)
//! - Level 3: Operator (can use /setblock, /testfor, /data, /clear, /clone)
//! - Level 4: Operator (console-level access, can execute any command)

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use uuid::Uuid;

/// Permission level for operators (0-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionLevel {
    /// Non-operator (default)
    None = 0,
    /// Operator level 1 - Can bypass spawn protection
    Level1 = 1,
    /// Operator level 2 - Can use /give, /summon, /tm, /effect
    Level2 = 2,
    /// Operator level 3 - Can use /setblock, /testfor, /data, /clear, /clone
    Level3 = 3,
    /// Operator level 4 - Console-level access
    Level4 = 4,
}

impl PermissionLevel {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            _ => Self::None,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }

    pub fn is_op(self) -> bool {
        self != Self::None
    }
}

impl Default for PermissionLevel {
    fn default() -> Self {
        Self::None
    }
}

/// Represents an operator entry in ops.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorEntry {
    /// The player's UUID.
    pub uuid: Uuid,
    /// The player's name.
    pub name: String,
    /// The permission level.
    #[serde(rename = "level")]
    pub permission_level: PermissionLevel,
    /// When the player was promoted to operator (timestamp in milliseconds since epoch).
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// Manages operator list and permissions.
pub struct PermissionManager {
    operators: Vec<OperatorEntry>,
    ops_file_path: String,
}

impl PermissionManager {
    /// Creates a new permission manager with the given config directory.
    pub fn new(config_dir: &str) -> Self {
        Self {
            operators: Vec::new(),
            ops_file_path: format!("{}/ops.json", config_dir),
        }
    }

    /// Loads operators from ops.json.
    pub fn load(config_dir: &str) -> Self {
        let ops_path = format!("{}/ops.json", config_dir);
        
        let operators = if Path::new(&ops_path).exists() {
            match fs::read_to_string(&ops_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                    log::warn!("Failed to parse ops.json: {}", e);
                    Vec::new()
                }),
                Err(e) => {
                    log::warn!("Failed to read ops.json: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        log::info!("Loaded {} operators", operators.len());

        Self { 
            operators, 
            ops_file_path: ops_path,
        }
    }

    /// Saves operators to ops.json.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(&self.operators)?;
        fs::write(&self.ops_file_path, content)?;
        Ok(())
    }

    /// Gets the permission level for a player by UUID.
    pub fn get_permission(&self, uuid: Uuid) -> PermissionLevel {
        self.operators
            .iter()
            .find(|op| op.uuid == uuid)
            .map(|op| op.permission_level)
            .unwrap_or_default()
    }

    /// Gets the permission level for a player by name.
    pub fn get_permission_by_name(&self, name: &str) -> PermissionLevel {
        self.operators
            .iter()
            .find(|op| op.name.eq_ignore_ascii_case(name))
            .map(|op| op.permission_level)
            .unwrap_or_default()
    }

    /// Checks if a player is an operator.
    pub fn is_op(&self, uuid: Uuid) -> bool {
        self.get_permission(uuid).is_op()
    }

    /// Adds or updates an operator.
    pub fn add_operator(&mut self, uuid: Uuid, name: String, level: PermissionLevel) {
        if let Some(existing) = self.operators.iter_mut().find(|op| op.uuid == uuid) {
            existing.name = name;
            existing.permission_level = level;
            existing.timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64
            );
        } else {
            self.operators.push(OperatorEntry {
                uuid,
                name,
                permission_level: level,
                timestamp: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64
                ),
            });
        }
    }

    /// Removes an operator.
    pub fn remove_operator(&mut self, uuid: Uuid) {
        self.operators.retain(|op| op.uuid != uuid);
    }

    /// Gets all operators.
    pub fn operators(&self) -> &[OperatorEntry] {
        &self.operators
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new("config")
    }
}