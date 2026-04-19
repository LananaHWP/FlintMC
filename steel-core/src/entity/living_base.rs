//! Shared fields for all living entities.
//!
//! Mirrors the fields that vanilla defines on `LivingEntity` (and `Entity` for
//! `invulnerableTime`). Entities that implement `LivingEntity` embed this
//! struct and expose it via `LivingEntity::living_base()`, just like
//! `EntityBase` is used for core `Entity` fields.

/// Duration in ticks of the death animation before entity removal.
pub const DEATH_DURATION: i32 = 20;

/// Common fields shared by all living entities.
///
/// **Deviation from vanilla:** In vanilla, `LivingEntity.dead` is only used by
/// non-player entities as a guard in `LivingEntity.die()`. `ServerPlayer.die()`
/// does NOT call `super.die()` and never sets `dead = true`. We use `dead` for
/// all living entities (including players) as a unified guard against duplicate
/// death processing, since it's cleaner than relying solely on `isRemoved()`.
pub struct LivingEntityBase {
    /// Whether the entity has been killed.
    ///
    /// See struct-level doc for vanilla deviation details.
    pub dead: bool,
    /// Remaining invulnerability ticks.
    pub invulnerable_time: i32,
    /// Last damage amount for invulnerability-frame comparison.
    pub last_hurt: f32,
    /// Ticks since the entity died. Incremented each tick while dead/dying.
    pub death_time: i32,
    /// Remaining ticks in love mode (-1 when not in love).
    /// This is the "inLove" field in vanilla - triggered when fed breeding food.
    pub love_mode_timer: i32,
    /// Entity ID of the breeding partner (when in love mode).
    pub love_partner_id: Option<i32>,
    /// Age in ticks. Negative = baby, positive = adult.
    /// mobs become adults when age >= 0.
    pub age: i32,
}

impl LivingEntityBase {
    /// Creates a new `LivingEntityBase` with default values (alive, no invulnerability, no hurt).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dead: false,
            invulnerable_time: 0,
            last_hurt: 0.0,
            death_time: 0,
            love_mode_timer: -1,
            love_partner_id: None,
            age: 0,
        }
    }

    /// Increments `death_time` by 1 and returns the new value.
    #[inline]
    pub const fn increment_death_time(&mut self) -> i32 {
        self.death_time += 1;
        self.death_time
    }

    /// Resets all death-related state back to alive defaults.
    #[inline]
    pub const fn reset_death_state(&mut self) {
        self.dead = false;
        self.death_time = 0;
        self.invulnerable_time = 0;
        self.last_hurt = 0.0;
    }

    /// Returns true if the mob is in love mode (can breed).
    #[inline]
    pub fn is_in_love_mode(&self) -> bool {
        self.love_mode_timer > 0
    }

    /// Sets the love mode timer (in ticks).
    #[inline]
    pub fn set_love_mode(&mut self, timer: i32, partner_id: Option<i32>) {
        self.love_mode_timer = timer;
        self.love_partner_id = partner_id;
    }

    /// Clears love mode.
    #[inline]
    pub fn clear_love_mode(&mut self) {
        self.love_mode_timer = -1;
        self.love_partner_id = None;
    }

    /// Decrements love mode timer by 1 if positive, returns remaining ticks.
    #[inline]
    pub fn decrement_love_mode(&mut self) -> i32 {
        if self.love_mode_timer > 0 {
            self.love_mode_timer -= 1;
            if self.love_mode_timer <= 0 {
                self.love_partner_id = None;
            }
        }
        self.love_mode_timer
    }

    /// Returns true if the mob is a baby (age < 0).
    #[inline]
    pub fn is_baby(&self) -> bool {
        self.age < 0
    }

    /// Returns true if the mob is an adult (age >= 0).
    #[inline]
    pub fn is_adult(&self) -> bool {
        self.age >= 0
    }

    /// Sets the age in ticks (negative = baby, 0 = adult).
    #[inline]
    pub fn set_age(&mut self, age: i32) {
        self.age = age;
    }
}

impl Default for LivingEntityBase {
    fn default() -> Self {
        Self::new()
    }
}
