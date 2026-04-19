use crate::entity::damage::DamageSource;
use steel_registry::damage_type::{DamageEffects, DamageScaling, DeathMessageType, DamageType};
use steel_registry::vanilla_attributes;
use steel_utils::Identifier;
use std::sync::LazyLock;

use crate::entity::attribute::{AttributeModifier, AttributeModifierOperation};
use crate::entity::LivingEntity;

pub const EFFECT_TICK_FREQUENCY_NORMAL: i32 = 1;
pub const EFFECT_TICK_FREQUENCY_SLOW: i32 = 2;
pub const EFFECT_TICK_FREQUENCY_FAST: i32 = 0;

pub const POISON_DAMAGE_INTERVAL: i32 = 25;
pub const WITHER_DAMAGE_INTERVAL: i32 = 40;

static POISON_DAMAGE_TYPE: LazyLock<DamageType> = LazyLock::new(|| DamageType {
    key: Identifier::vanilla_static("poison"),
    message_id: "poison",
    scaling: DamageScaling::WhenCausedByLivingNonPlayer,
    exhaustion: 0.0,
    effects: DamageEffects::Hurt,
    death_message_type: DeathMessageType::Default,
});
static WITHER_DAMAGE_TYPE: LazyLock<DamageType> = LazyLock::new(|| DamageType {
    key: Identifier::vanilla_static("wither"),
    message_id: "wither",
    scaling: DamageScaling::Always,
    exhaustion: 0.0,
    effects: DamageEffects::Hurt,
    death_message_type: DeathMessageType::Default,
});
static MAGIC_DAMAGE_TYPE: LazyLock<DamageType> = LazyLock::new(|| DamageType {
    key: Identifier::vanilla_static("magic"),
    message_id: "magic",
    scaling: DamageScaling::Always,
    exhaustion: 0.0,
    effects: DamageEffects::Hurt,
    death_message_type: DeathMessageType::Default,
});

pub fn calculate_ticks_from_duration(effect_id: i32, duration: i32, amplifier: i32) -> i32 {
    if duration <= 0 {
        return 0;
    }

    match effect_id {
        9 => {
            if duration >= 50 {
                duration * 20 / (50 * (1 << amplifier))
            } else {
                0
            }
        }
        | 18 | 19 => {
            let tick_frequency = if effect_id == 18 {
                POISON_DAMAGE_INTERVAL
            } else {
                WITHER_DAMAGE_INTERVAL
            };
            let calculated = (duration / tick_frequency) * tick_frequency;
            if calculated > 0 {
                calculated
            } else {
                tick_frequency
            }
        }
        | 3 => duration / (1 << amplifier),
        | 2 => duration / (1 << (2.min(amplifier))),
        | 1 => duration / (1 << amplifier),
        | 7 => duration / (if amplifier >= 2 { 1 } else { 1 << amplifier }),
        | 6 => duration,
        | _ => duration,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Speed,
    Slowness,
    Haste,
    MiningFatigue,
    Strength,
    InstantHealth,
    InstantDamage,
    JumpBoost,
    Nausea,
    Regeneration,
    Resistance,
    FireResistance,
    WaterBreathing,
    Invisibility,
    Blindness,
    NightVision,
    Hunger,
    Weakness,
    Poison,
    Wither,
    HealthBoost,
    Absorption,
    Saturation,
    Glowing,
    Levitation,
    Luck,
    Unluck,
    SlowFalling,
    ConduitPower,
    DolphinsGrace,
    BadOmen,
    HeroOfTheVillage,
    Darkness,
    TrialOmen,
    RaidOmen,
    WindCharged,
    Weaving,
    Oozing,
    Infested,
    BreathOfTheNautilus,
}

impl StatusEffect {
    pub fn from_mob_effect_id(id: i32) -> Option<Self> {
        Some(match id {
            0 => StatusEffect::Speed,
            1 => StatusEffect::Slowness,
            2 => StatusEffect::Haste,
            3 => StatusEffect::MiningFatigue,
            4 => StatusEffect::Strength,
            5 => StatusEffect::InstantHealth,
            6 => StatusEffect::InstantDamage,
            7 => StatusEffect::JumpBoost,
            8 => StatusEffect::Nausea,
            9 => StatusEffect::Regeneration,
            10 => StatusEffect::Resistance,
            11 => StatusEffect::FireResistance,
            12 => StatusEffect::WaterBreathing,
            13 => StatusEffect::Invisibility,
            14 => StatusEffect::Blindness,
            15 => StatusEffect::NightVision,
            16 => StatusEffect::Hunger,
            17 => StatusEffect::Weakness,
            18 => StatusEffect::Poison,
            19 => StatusEffect::Wither,
            20 => StatusEffect::HealthBoost,
            21 => StatusEffect::Absorption,
            22 => StatusEffect::Saturation,
            23 => StatusEffect::Glowing,
            24 => StatusEffect::Levitation,
            25 => StatusEffect::Luck,
            26 => StatusEffect::Unluck,
            27 => StatusEffect::SlowFalling,
            28 => StatusEffect::ConduitPower,
            29 => StatusEffect::DolphinsGrace,
            30 => StatusEffect::BadOmen,
            31 => StatusEffect::HeroOfTheVillage,
            32 => StatusEffect::Darkness,
            33 => StatusEffect::TrialOmen,
            34 => StatusEffect::RaidOmen,
            35 => StatusEffect::WindCharged,
            36 => StatusEffect::Weaving,
            37 => StatusEffect::Oozing,
            38 => StatusEffect::Infested,
            39 => StatusEffect::BreathOfTheNautilus,
            _ => return None,
        })
    }

    pub fn from_key(key: &Identifier) -> Option<Self> {
        use steel_utils::Identifier;
        let path = key.path.as_ref();
        let ns = key.namespace.as_ref();
        if ns != "minecraft" {
            return None;
        }
        match path {
            "speed" => Some(StatusEffect::Speed),
            "slowness" => Some(StatusEffect::Slowness),
            "haste" => Some(StatusEffect::Haste),
            "mining_fatigue" => Some(StatusEffect::MiningFatigue),
            "strength" => Some(StatusEffect::Strength),
            "instant_health" => Some(StatusEffect::InstantHealth),
            "instant_damage" => Some(StatusEffect::InstantDamage),
            "jump_boost" => Some(StatusEffect::JumpBoost),
            "nausea" => Some(StatusEffect::Nausea),
            "regeneration" => Some(StatusEffect::Regeneration),
            "resistance" => Some(StatusEffect::Resistance),
            "fire_resistance" => Some(StatusEffect::FireResistance),
            "water_breathing" => Some(StatusEffect::WaterBreathing),
            "invisibility" => Some(StatusEffect::Invisibility),
            "blindness" => Some(StatusEffect::Blindness),
            "night_vision" => Some(StatusEffect::NightVision),
            "hunger" => Some(StatusEffect::Hunger),
            "weakness" => Some(StatusEffect::Weakness),
            "poison" => Some(StatusEffect::Poison),
            "wither" => Some(StatusEffect::Wither),
            "health_boost" => Some(StatusEffect::HealthBoost),
            "absorption" => Some(StatusEffect::Absorption),
            "saturation" => Some(StatusEffect::Saturation),
            "glowing" => Some(StatusEffect::Glowing),
            "levitation" => Some(StatusEffect::Levitation),
            "luck" => Some(StatusEffect::Luck),
            "unluck" => Some(StatusEffect::Unluck),
            "slow_falling" => Some(StatusEffect::SlowFalling),
            "conduit_power" => Some(StatusEffect::ConduitPower),
            "dolphins_grace" => Some(StatusEffect::DolphinsGrace),
            "bad_omen" => Some(StatusEffect::BadOmen),
            "hero_of_the_village" => Some(StatusEffect::HeroOfTheVillage),
            "darkness" => Some(StatusEffect::Darkness),
            "trial_omen" => Some(StatusEffect::TrialOmen),
            "raid_omen" => Some(StatusEffect::RaidOmen),
            "wind_charged" => Some(StatusEffect::WindCharged),
            "weaving" => Some(StatusEffect::Weaving),
            "oozing" => Some(StatusEffect::Oozing),
            "infested" => Some(StatusEffect::Infested),
            "breath_of_the_nautilus" => Some(StatusEffect::BreathOfTheNautilus),
            _ => None,
        }
    }

    pub fn mob_effect_id(&self) -> i32 {
        match self {
            StatusEffect::Speed => 0,
            StatusEffect::Slowness => 1,
            StatusEffect::Haste => 2,
            StatusEffect::MiningFatigue => 3,
            StatusEffect::Strength => 4,
            StatusEffect::InstantHealth => 5,
            StatusEffect::InstantDamage => 6,
            StatusEffect::JumpBoost => 7,
            StatusEffect::Nausea => 8,
            StatusEffect::Regeneration => 9,
            StatusEffect::Resistance => 10,
            StatusEffect::FireResistance => 11,
            StatusEffect::WaterBreathing => 12,
            StatusEffect::Invisibility => 13,
            StatusEffect::Blindness => 14,
            StatusEffect::NightVision => 15,
            StatusEffect::Hunger => 16,
            StatusEffect::Weakness => 17,
            StatusEffect::Poison => 18,
            StatusEffect::Wither => 19,
            StatusEffect::HealthBoost => 20,
            StatusEffect::Absorption => 21,
            StatusEffect::Saturation => 22,
            StatusEffect::Glowing => 23,
            StatusEffect::Levitation => 24,
            StatusEffect::Luck => 25,
            StatusEffect::Unluck => 26,
            StatusEffect::SlowFalling => 27,
            StatusEffect::ConduitPower => 28,
            StatusEffect::DolphinsGrace => 29,
            StatusEffect::BadOmen => 30,
            StatusEffect::HeroOfTheVillage => 31,
            StatusEffect::Darkness => 32,
            StatusEffect::TrialOmen => 33,
            StatusEffect::RaidOmen => 34,
            StatusEffect::WindCharged => 35,
            StatusEffect::Weaving => 36,
            StatusEffect::Oozing => 37,
            StatusEffect::Infested => 38,
            StatusEffect::BreathOfTheNautilus => 39,
        }
    }

    pub fn is_instant(&self) -> bool {
        matches!(
            self,
            StatusEffect::InstantHealth | StatusEffect::InstantDamage | StatusEffect::Saturation
        )
    }

    pub fn get_duration(&self, duration: i32, amplifier: i32) -> i32 {
        if duration <= 0 {
            return 0;
        }

        if self.is_instant() {
            return 1;
        }

        calculate_ticks_from_duration(self.mob_effect_id(), duration, amplifier)
    }

    pub fn apply(&self, entity: &dyn LivingEntity, amplifier: i32, duration: i32) {
        if entity.is_dead_or_dying() {
            return;
        }

        match self {
            StatusEffect::Speed => {
                let amount = 0.2 * (1.0 + amplifier as f64);
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.speed"),
                    amount,
                    operation: AttributeModifierOperation::AddMultipliedTotal,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::MOVEMENT_SPEED, modifier, true);
            }
            StatusEffect::Slowness => {
                let amount = -0.15 * (1.0 + amplifier as f64);
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.slowness"),
                    amount,
                    operation: AttributeModifierOperation::AddMultipliedTotal,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::MOVEMENT_SPEED, modifier, true);
            }
            StatusEffect::Haste => {
                let amount = 0.2 * (1.0 + amplifier as f64);
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.haste"),
                    amount,
                    operation: AttributeModifierOperation::AddMultipliedTotal,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::ATTACK_SPEED, modifier, true);
            }
            StatusEffect::MiningFatigue => {
                let amount = -0.3 * (1.0 + amplifier as f64);
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.mining_fatigue"),
                    amount,
                    operation: AttributeModifierOperation::AddMultipliedTotal,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::BLOCK_BREAK_SPEED, modifier, true);
            }
            StatusEffect::Strength => {
                let amount = 1.3 * (1.0 + amplifier as f64) - 1.0;
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.strength"),
                    amount,
                    operation: AttributeModifierOperation::AddMultipliedTotal,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::ATTACK_DAMAGE, modifier, true);
            }
            StatusEffect::InstantHealth => {
                let heal_amount = (4 << amplifier) as f32;
                entity.heal(heal_amount);
            }
            StatusEffect::InstantDamage => {
                let damage_amount = (6 << amplifier) as f32;
                let source = DamageSource::environment(&MAGIC_DAMAGE_TYPE);
                LivingEntity::hurt(entity, source, damage_amount);
            }
            StatusEffect::JumpBoost => {
                let amount = 0.5 + 0.25 * amplifier as f64;
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.jump_boost"),
                    amount,
                    operation: AttributeModifierOperation::AddValue,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::GRAVITY, modifier, true);
            }
            StatusEffect::Regeneration | StatusEffect::Resistance => {}
            StatusEffect::FireResistance => {}
            StatusEffect::WaterBreathing => {}
            StatusEffect::Invisibility => {}
            StatusEffect::Blindness => {}
            StatusEffect::NightVision => {}
            StatusEffect::Hunger => {}
            StatusEffect::Weakness => {
                let amount = -4.0 * (1.0 + amplifier as f64);
                let modifier = AttributeModifier {
                    id: Identifier::vanilla_static("effect.minecraft.weakness"),
                    amount,
                    operation: AttributeModifierOperation::AddValue,
                };
                let mut attrs = entity.attributes().lock();
                attrs.set_modifier(vanilla_attributes::ATTACK_DAMAGE, modifier, true);
            }
            StatusEffect::Poison | StatusEffect::Wither => {}
            StatusEffect::HealthBoost => {
                let bonus = (amplifier as f32 + 1.0) * 4.0;
                let base_max = entity.attributes().lock().get_base_value(vanilla_attributes::MAX_HEALTH);
                let new_max = base_max.map(|b| b + bonus as f64).unwrap_or(20.0);
                entity.attributes().lock().set_base_value(vanilla_attributes::MAX_HEALTH, new_max);
                let current_health = entity.get_health();
                if current_health as f64 > new_max {
                    entity.set_health(current_health);
                }
            }
            StatusEffect::Absorption => {
                let amount = (amplifier as f32 + 1.0) * 4.0;
                let current = entity.get_absorption_amount();
                entity.set_absorption_amount(current + amount);
            }
            StatusEffect::Saturation => {}
            StatusEffect::Glowing => {}
            StatusEffect::Levitation => {}
            StatusEffect::Luck => {}
            StatusEffect::Unluck => {}
            StatusEffect::SlowFalling => {}
            StatusEffect::ConduitPower => {}
            StatusEffect::DolphinsGrace => {}
            StatusEffect::BadOmen => {}
            StatusEffect::HeroOfTheVillage => {}
            _ => {}
        }
    }

    pub fn tick(&self, entity: &dyn LivingEntity, amplifier: i32, duration: i32, ambient: bool) {
        if entity.is_dead_or_dying() {
            return;
        }

        match self {
            StatusEffect::Regeneration => {
                let tick_rate = (20_i32 * (1_i32 << amplifier)).max(1);
                if duration % tick_rate == 0 {
                    entity.heal(1.0_f32);
                }
            }
            StatusEffect::Poison => {
                if !entity.is_poison_source_safe() {
                    return;
                }
                if duration % POISON_DAMAGE_INTERVAL == 0 {
                    let damage = (1_i32 << amplifier).max(1) as f32;
                    let source = DamageSource::environment(&POISON_DAMAGE_TYPE);
                    LivingEntity::hurt(entity, source, damage);
                }
            }
            StatusEffect::Wither => {
                if duration % WITHER_DAMAGE_INTERVAL == 0 {
                    let damage = (1_i32 << amplifier).max(1) as f32;
                    let source = DamageSource::environment(&WITHER_DAMAGE_TYPE);
                    LivingEntity::hurt(entity, source, damage);
                }
            }
            _ => {}
        }
    }

    pub fn on_effect_end(&self, entity: &dyn LivingEntity) {
        match self {
            StatusEffect::Speed => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::MOVEMENT_SPEED,
                    &Identifier::vanilla_static("effect.minecraft.speed"),
                );
            }
            StatusEffect::Slowness => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::MOVEMENT_SPEED,
                    &Identifier::vanilla_static("effect.minecraft.slowness"),
                );
            }
            StatusEffect::Haste => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::ATTACK_SPEED,
                    &Identifier::vanilla_static("effect.minecraft.haste"),
                );
            }
            StatusEffect::MiningFatigue => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::BLOCK_BREAK_SPEED,
                    &Identifier::vanilla_static("effect.minecraft.mining_fatigue"),
                );
            }
            StatusEffect::Strength => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::ATTACK_DAMAGE,
                    &Identifier::vanilla_static("effect.minecraft.strength"),
                );
            }
            StatusEffect::Weakness => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::ATTACK_DAMAGE,
                    &Identifier::vanilla_static("effect.minecraft.weakness"),
                );
            }
            StatusEffect::JumpBoost => {
                let mut attrs = entity.attributes().lock();
                attrs.remove_modifier(
                    vanilla_attributes::GRAVITY,
                    &Identifier::vanilla_static("effect.minecraft.jump_boost"),
                );
            }
            StatusEffect::HealthBoost => {
                let base = entity
                    .attributes()
                    .lock()
                    .get_base_value(vanilla_attributes::MAX_HEALTH)
                    .unwrap_or(20.0);
                let new_max = (base - (entity.get_max_health() as f64 - base).max(0.0)).max(20.0);
                entity.attributes().lock().set_base_value(vanilla_attributes::MAX_HEALTH, new_max);
            }
            _ => {}
        }
    }

    pub fn has_display_icon(&self) -> bool {
        true
    }

    pub fn is_beneficial(&self) -> bool {
        !matches!(
            self,
            StatusEffect::Slowness
                | StatusEffect::MiningFatigue
                | StatusEffect::Nausea
                | StatusEffect::Blindness
                | StatusEffect::Hunger
                | StatusEffect::Weakness
                | StatusEffect::Poison
                | StatusEffect::Wither
                | StatusEffect::Darkness
                | StatusEffect::BadOmen
                | StatusEffect::TrialOmen
                | StatusEffect::Weaving
                | StatusEffect::Oozing
                | StatusEffect::Infested
                | StatusEffect::WindCharged
        )
    }
}