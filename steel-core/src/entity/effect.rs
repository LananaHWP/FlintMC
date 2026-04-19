use steel_registry::mob_effect::{MobEffect, MobEffectRef, MobEffectCategory};
use steel_registry::vanilla_mob_effects;
use steel_registry::REGISTRY;
use steel_utils::Identifier;

use crate::entity::attribute::{AttributeMap, AttributeModifier, AttributeModifierOperation};
use crate::entity::LivingEntity;
use crate::entity::damage::DamageSource;
use glam::DVec3;
use std::sync::{Arc, Weak};
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_damage_types;
use std::collections::HashMap;

pub mod active_effect;
pub mod effect_type;

pub use active_effect::{ActiveEffect, ActiveEffectMap};
pub use effect_type::{StatusEffect, calculate_ticks_from_duration};

pub fn by_id(id: i32) -> Option<StatusEffect> {
    StatusEffect::from_mob_effect_id(id)
}

pub fn by_key(key: &Identifier) -> Option<StatusEffect> {
    StatusEffect::from_key(key)
}