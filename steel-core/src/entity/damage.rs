//! Damage source system.

use glam::DVec3;
use steel_registry::damage_type::{DamageEffects, DamageScaling, DamageType};

/// Describes how an entity was damaged.
#[derive(Debug, Clone)]
pub struct DamageSource {
    /// The damage type registry entry.
    pub damage_type: &'static DamageType,
    /// The entity ultimately responsible (e.g. the shooter for projectiles).
    pub causing_entity_id: Option<i32>,
    /// The entity that directly dealt the damage (e.g. the projectile itself).
    pub direct_entity_id: Option<i32>,
    /// Source position (for explosions, etc.).
    pub source_position: Option<DVec3>,
}

impl DamageSource {
    /// Environmental damage with no entity or position context (void, starvation, etc.).
    #[must_use]
    pub const fn environment(damage_type: &'static DamageType) -> Self {
        Self {
            damage_type,
            causing_entity_id: None,
            direct_entity_id: None,
            source_position: None,
        }
    }

    /// Damage caused by an entity (mob attack).
    #[must_use]
    pub const fn entity(damage_type: &'static DamageType, entity_id: i32) -> Self {
        Self {
            damage_type,
            causing_entity_id: Some(entity_id),
            direct_entity_id: Some(entity_id),
            source_position: None,
        }
    }

    /// Generic mob attack damage (fallback when no specific damage type applies).
    #[must_use]
    pub fn generic_mob_attack() -> Self {
        static MOB_ATTACK_TYPE: DamageType = DamageType {
            key: steel_utils::Identifier::new_static("minecraft", "mob_attack"),
            message_id: "attack",
            scaling: DamageScaling::Always,
            exhaustion: 0.1,
            effects: DamageEffects::Hurt,
            death_message_type: steel_registry::damage_type::DeathMessageType::Default,
        };
        Self {
            damage_type: &MOB_ATTACK_TYPE,
            causing_entity_id: None,
            direct_entity_id: None,
            source_position: None,
        }
    }

    /// Whether this damage bypasses creative/spectator invulnerability.
    #[must_use]
    pub fn bypasses_invulnerability(&self) -> bool {
        matches!(&*self.damage_type.key.path, "out_of_world" | "generic_kill")
    }

    /// Whether this damage bypasses the invulnerability cooldown timer.
    #[must_use]
    pub const fn bypasses_cooldown(&self) -> bool {
        false
    }

    /// Whether this damage scales with world difficulty.
    #[must_use]
    pub const fn scales_with_difficulty(&self) -> bool {
        match self.damage_type.scaling {
            DamageScaling::Never => false,
            DamageScaling::Always | DamageScaling::WhenCausedByLivingNonPlayer => true,
        }
    }
}