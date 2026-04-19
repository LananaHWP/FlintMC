use rustc_hash::FxHashMap;
use steel_utils::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobEffectCategory {
    Beneficial,
    Harmful,
    Neutral,
}

#[derive(Debug)]
pub struct MobEffect {
    pub id: i32,
    pub key: Identifier,
    pub translation_key: &'static str,
    pub category: MobEffectCategory,
    pub color: i32,
}

impl MobEffect {
    pub const fn new(
        id: i32,
        namespace: &'static str,
        path: &'static str,
        translation_key: &'static str,
        category: MobEffectCategory,
        color: i32,
    ) -> Self {
        Self {
            id,
            key: Identifier::new_static(namespace, path),
            translation_key,
            category,
            color,
        }
    }
}

pub type MobEffectRef = &'static MobEffect;

pub struct MobEffectRegistry {
    effects_by_id: Vec<MobEffectRef>,
    effects_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl Default for MobEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MobEffectRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            effects_by_id: Vec::new(),
            effects_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    pub fn register(&mut self, effect: MobEffectRef) {
        assert!(
            self.allows_registering,
            "Cannot register mob effects after the registry has been frozen"
        );
        let idx = self.effects_by_id.len();
        self.effects_by_key.insert(effect.key.clone(), idx);
        self.effects_by_id.push(effect);
    }

    pub fn freeze(&mut self) {
        self.allows_registering = false;
    }

    pub fn by_id(&self, id: usize) -> Option<MobEffectRef> {
        self.effects_by_id.get(id).copied()
    }

    pub fn by_key(&self, key: &Identifier) -> Option<MobEffectRef> {
        self.effects_by_key
            .get(key)
            .and_then(|&idx| self.effects_by_id.get(idx).copied())
    }

    pub fn len(&self) -> usize {
        self.effects_by_id.len()
    }
}

crate::impl_registry!(
    MobEffectRegistry,
    MobEffect,
    effects_by_id,
    effects_by_key,
    mob_effects
);