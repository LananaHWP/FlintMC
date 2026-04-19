//! Spawn protection logic for preventing PvP and building near world spawn.
//!
//! In vanilla, there's a spawn protection zone (default 16 blocks radius) around the world spawn
//! point where:
//! - Block breaking is restricted (unless player is in creative or has operator permissions)
//! - Block placement is restricted (unless player is in creative or has operator permissions)
//! - PvP damage is prevented

use crate::player::Player;
use crate::world::World;
use std::sync::Arc;

/// Default spawn protection radius in blocks.
/// Vanilla default is 16 blocks.
pub const DEFAULT_SPAWN_PROTECTION_RADIUS: i32 = 16;

/// Checks if an action is allowed within the spawn protection zone.
/// Returns true if the action should be allowed, false if blocked.
pub fn check_spawn_protection(
    player: &Player,
    world: &Arc<World>,
    _action: SpawnProtectedAction,
) -> bool {
    // Operators can bypass spawn protection
    if player.is_op() {
        return true;
    }

    // Creative mode bypasses spawn protection
    if player.game_mode.load() == steel_utils::types::GameType::Creative {
        return true;
    }

    // Spectator mode can also bypass (they can't interact anyway)
    if player.game_mode.load() == steel_utils::types::GameType::Spectator {
        return true;
    }

    let pos = player.position.lock();
    world.is_within_spawn_protection(pos.x, pos.z)
}

/// Spawn-protected action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnProtectedAction {
    /// Breaking a block.
    BlockBreak,
    /// Placing a block.
    BlockPlace,
    /// PvP damage attempt.
    PvPDamage,
}

impl SpawnProtectedAction {
    /// Returns true if the action type is protected in spawn zone.
    pub fn is_protected(self) -> bool {
        matches!(self, Self::BlockBreak | Self::BlockPlace | Self::PvPDamage)
    }
}