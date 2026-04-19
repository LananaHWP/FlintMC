//! Projectile entity base functionality.
//!
//! This module provides common functionality for all projectile entities.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use steel_registry::blocks::shapes::AABBd;
use steel_registry::entity_types::EntityTypeRef;
use steel_utils::locks::SyncMutex;
use uuid::Uuid;

use crate::entity::damage::DamageSource;
use crate::entity::{Entity, EntityBase, LivingEntity, RemovalReason};
use crate::world::World;

const DEFAULT_GRAVITY: f64 = 0.05;
const DRAG: f64 = 0.99;

/// Maximum lifetime in ticks (60 seconds).
const MAX_LIFETIME: i32 = 1200;

/// Velocity sync threshold squared.
const VELOCITY_SYNC_THRESHOLD: f64 = 1.0e-7;
const POSITION_SYNC_THRESHOLD: f64 = 7.629_394_5e-6;

pub mod projectile {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ProjectileState {
        Flying,
        Stuck,
        Removed,
    }
}
use projectile::ProjectileState;

pub struct ProjectileStateContainer {
    pub state: AtomicI32,
    pub lifetime: AtomicI32,
    pub last_sent_velocity: SyncMutex<DVec3>,
    pub last_sent_position: SyncMutex<DVec3>,
    pub needs_sync: AtomicBool,
}

impl ProjectileStateContainer {
    pub fn new() -> Self {
        Self {
            state: AtomicI32::new(ProjectileState::Flying as i32),
            lifetime: AtomicI32::new(0),
            last_sent_velocity: SyncMutex::new(DVec3::ZERO),
            last_sent_position: SyncMutex::new(DVec3::ZERO),
            needs_sync: AtomicBool::new(false),
        }
    }

    pub fn get_state(&self) -> ProjectileState {
        match self.state.load(Ordering::Relaxed) {
            0 => ProjectileState::Flying,
            1 => ProjectileState::Stuck,
            _ => ProjectileState::Removed,
        }
    }

    pub fn set_state(&self, state: ProjectileState) {
        self.state.store(state as i32, Ordering::Relaxed);
    }
}

impl Default for ProjectileStateContainer {
    fn default() -> Self {
        Self::new()
    }
}