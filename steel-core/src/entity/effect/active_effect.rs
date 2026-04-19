use super::effect_type::StatusEffect;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ActiveEffect {
    pub effect: StatusEffect,
    pub amplifier: i32,
    pub duration: i32,
    pub ambient: bool,
}

impl ActiveEffect {
    pub fn new(effect: StatusEffect, amplifier: i32, duration: i32) -> Self {
        Self {
            effect,
            amplifier,
            duration,
            ambient: false,
        }
    }

    pub fn with_ambient(mut self, ambient: bool) -> Self {
        self.ambient = ambient;
        self
    }

    pub fn tick(&mut self) {
        if self.duration > 0 {
            self.duration -= 1;
        }
    }

    pub fn is_expired(&self) -> bool {
        self.duration <= 0
    }

    pub fn get_display_duration(&self) -> i32 {
        if self.duration <= 0 {
            return 0;
        }
        if self.duration >= 100 {
            return self.duration / 20;
        }
        1
    }
}

#[derive(Debug, Default)]
pub struct ActiveEffectMap {
    effects: HashMap<StatusEffect, ActiveEffect>,
}

impl ActiveEffectMap {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    pub fn add_effect(&mut self, effect: StatusEffect, amplifier: i32, duration: i32) {
        let actual_duration = effect.get_duration(duration, amplifier);

        if effect.is_instant() {
            self.effects.insert(effect, ActiveEffect::new(effect, amplifier, 1));
        } else {
            let display_duration = if actual_duration > 0 {
                actual_duration
            } else {
                duration
            };
            self.effects
                .insert(effect, ActiveEffect::new(effect, amplifier, display_duration));
        }
    }

    pub fn remove_effect(&mut self, effect: StatusEffect) -> Option<ActiveEffect> {
        self.effects.remove(&effect)
    }

    pub fn get(&self, effect: StatusEffect) -> Option<&ActiveEffect> {
        self.effects.get(&effect)
    }

    pub fn has_effect(&self, effect: StatusEffect) -> bool {
        self.effects.contains_key(&effect)
    }

    pub fn tick(&mut self) {
        for (_, active) in self.effects.iter_mut() {
            active.tick();
        }
        self.effects.retain(|_, effect| !effect.is_expired());
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StatusEffect, &ActiveEffect)> {
        self.effects.iter()
    }

    pub fn get_all(&self) -> Vec<&ActiveEffect> {
        self.effects.values().collect()
    }

    pub fn has_beneficial_effect(&self) -> bool {
        self.effects.values().any(|e| e.effect.is_beneficial())
    }

    pub fn has_non_beneficial_effect(&self) -> bool {
        self.effects.values().any(|e| !e.effect.is_beneficial())
    }
}