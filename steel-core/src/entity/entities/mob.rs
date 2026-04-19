//! Mob entity implementation for all living mobs.
//!
//! This module provides a common implementation for all creature, monster,
//! ambient, and water mobs. The specific mob type is determined by the entity type
//! registered in the entity registry.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use glam::DVec3;
use steel_registry::blocks::shapes::AABBd;
use steel_registry::entity_types::EntityTypeRef;
use steel_utils::locks::SyncMutex;
use uuid::Uuid;

use crate::entity::ai::{MobAI, mobs};
use crate::entity::ai::RandomStrollGoal;
use crate::entity::damage::DamageSource;
use crate::entity::effect::ActiveEffectMap;

use crate::entity::{Entity, EntityBase, LivingEntity, LivingEntityBase, RemovalReason};
use crate::physics::MoverType;
use crate::world::World;
use steel_registry::vanilla_attributes;

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::NbtCompound;

use crate::entity::attribute::AttributeMap;

const DEFAULT_HEALTH: f32 = 10.0;
const DEFAULT_GRAVITY: f64 = 0.08;

pub struct MobEntity {
    base: EntityBase,
    entity_type_ref: &'static steel_registry::entity_types::EntityType,
    health: AtomicI32,
    living_base: SyncMutex<LivingEntityBase>,
    attributes: SyncMutex<AttributeMap>,
    velocity: SyncMutex<DVec3>,
    rotation: AtomicCell<(f32, f32)>,
    on_ground: AtomicBool,
    hurt_time: AtomicI32,
    ai: SyncMutex<MobAI>,
    active_effects: SyncMutex<ActiveEffectMap>,
}

impl MobEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_type: EntityTypeRef,
        id: i32,
        position: DVec3,
        world: Weak<World>,
    ) -> Self {
        let attributes = AttributeMap::new_for_entity(entity_type);
        
        let health = entity_type.default_attributes.iter()
            .find(|(n, _)| *n == "max_health")
            .map(|(_, v)| *v)
            .unwrap_or(DEFAULT_HEALTH as f64) as i32;

        let ai = MobAI::new();
        {
            let mut goals = ai.goal_selector().lock();
            let stroll_goal = Box::new(RandomStrollGoal::new(1.0));
            goals.add_goal(stroll_goal, 2);
        }
        mobs::apply_mob_ai(&ai, &entity_type.key);

        Self {
            base: EntityBase::new(id, position, world),
            entity_type_ref: entity_type,
            health: AtomicI32::new(health as i32),
            living_base: SyncMutex::new(LivingEntityBase::new()),
            attributes: SyncMutex::new(attributes),
            velocity: SyncMutex::new(DVec3::ZERO),
            rotation: AtomicCell::new((0.0, 0.0)),
            on_ground: AtomicBool::new(false),
            hurt_time: AtomicI32::new(0),
            ai: SyncMutex::new(ai),
            active_effects: SyncMutex::new(ActiveEffectMap::new()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_saved(
        entity_type: EntityTypeRef,
        id: i32,
        position: DVec3,
        uuid: Uuid,
        velocity: DVec3,
        rotation: (f32, f32),
        on_ground: bool,
        world: Weak<World>,
    ) -> Self {
        let attributes = AttributeMap::new_for_entity(entity_type);

        let ai = MobAI::new();
        {
            let mut goals = ai.goal_selector().lock();
            let stroll_goal = Box::new(RandomStrollGoal::new(1.0));
            goals.add_goal(stroll_goal, 2);
        }
        mobs::apply_mob_ai(&ai, &entity_type.key);

        Self {
            base: EntityBase::with_uuid(id, uuid, position, world),
            entity_type_ref: entity_type,
            health: AtomicI32::new(10),
            living_base: SyncMutex::new(LivingEntityBase::new()),
            attributes: SyncMutex::new(attributes),
            velocity: SyncMutex::new(velocity),
            rotation: AtomicCell::new(rotation),
            on_ground: AtomicBool::new(on_ground),
            hurt_time: AtomicI32::new(0),
            ai: SyncMutex::new(ai),
            active_effects: SyncMutex::new(ActiveEffectMap::new()),
        }
    }
    
    pub fn get_entity_type_ref(&self) -> EntityTypeRef {
        self.entity_type_ref
    }
}

impl Entity for MobEntity {
    fn base(&self) -> Option<&EntityBase> {
        Some(&self.base)
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type_ref
    }

    fn bounding_box(&self) -> AABBd {
        let pos = self.position();
        let dims = self.entity_type().dimensions;
        let half_width = f64::from(dims.width) / 2.0;
        let height = f64::from(dims.height);
        AABBd {
            min_x: pos.x - half_width,
            min_y: pos.y,
            min_z: pos.z - half_width,
            max_x: pos.x + half_width,
            max_y: pos.y + height,
            max_z: pos.z + half_width,
        }
    }

    fn tick(&self) {
        let hurt_time = self.hurt_time.load(Ordering::Relaxed);
        if hurt_time > 0 {
            self.hurt_time.fetch_sub(1, Ordering::Relaxed);
        }

        self.ai.lock().tick(self);

        self.apply_gravity();

        if let Some(result) = self.do_move(MoverType::SelfMovement) {
            self.set_on_ground(result.on_ground);
        }
    }

    fn get_default_gravity(&self) -> f64 {
        let gravity = self.attributes()
            .lock()
            .get_value(vanilla_attributes::GRAVITY)
            .unwrap_or(DEFAULT_GRAVITY);
        gravity
    }

    fn rotation(&self) -> (f32, f32) {
        self.rotation.load()
    }

    fn velocity(&self) -> DVec3 {
        *self.velocity.lock()
    }

    fn set_velocity(&self, velocity: DVec3) {
        *self.velocity.lock() = velocity;
    }

    fn on_ground(&self) -> bool {
        self.on_ground.load(Ordering::Relaxed)
    }

    fn set_on_ground(&self, on_ground: bool) {
        self.on_ground.store(on_ground, Ordering::Relaxed);
    }

    fn as_living(self: Arc<Self>) -> Option<Arc<dyn crate::entity::LivingEntity>> where Self: Sized {
        Some(self)
    }

    fn as_living_ref(&self) -> Option<&dyn crate::entity::LivingEntity> {
        Some(self)
    }

    fn hurt(&self, _source: &DamageSource, amount: f32) -> bool {
        if self.living_base.lock().invulnerable_time > 0 {
            return false;
        }

        let mut living_base = self.living_base.lock();
        living_base.invulnerable_time = 20;
        living_base.last_hurt = amount;

        let health = self.get_health();
        let new_health = (health - amount).max(0.0);
        self.set_health(new_health);

        self.hurt_time.store(10, Ordering::Relaxed);

        if new_health <= 0.0 {
            living_base.dead = true;
        }

        true
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let health = self.get_health();
        nbt.insert("Health", health);
        nbt.insert("AbsorptionAmount", 0.0f32);
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt: NbtCompoundView<'_, '_> = nbt.into();

        if let Some(health) = nbt.float("Health") {
            self.set_health(health);
        }
    }
}

impl LivingEntity for MobEntity {
    fn attributes(&self) -> &SyncMutex<AttributeMap> {
        &self.attributes
    }

    fn get_health(&self) -> f32 {
        self.health.load(Ordering::Relaxed) as f32
    }

    fn set_health(&self, health: f32) {
        self.health.store(health as i32, Ordering::Relaxed);
    }

    fn hurt(&self, source: DamageSource, amount: f32) {
        if source.bypasses_invulnerability() {
            let new_health = (self.get_health() - amount).max(0.0);
            self.set_health(new_health);
            if new_health <= 0.0 {
                self.set_removed(RemovalReason::Killed);
            }
        }
    }

    fn living_base(&self) -> &SyncMutex<LivingEntityBase> {
        &self.living_base
    }

    fn get_absorption_amount(&self) -> f32 {
        0.0
    }

    fn set_absorption_amount(&self, _amount: f32) {}

    fn set_sprinting(&self, _sprinting: bool) {}

    fn get_speed(&self) -> f32 {
        self.attributes()
            .lock()
            .get_value(vanilla_attributes::MOVEMENT_SPEED)
            .unwrap_or(0.25) as f32
    }

    fn set_speed(&self, speed: f32) {
        self.attributes()
            .lock()
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, f64::from(speed));
    }

    fn set_y_velocity(&self, velocity: f64) {
        let mut vel = *self.velocity.lock();
        vel.y = velocity;
        *self.velocity.lock() = vel;
    }

    fn active_effects(&self) -> &SyncMutex<ActiveEffectMap> {
        &self.active_effects
    }
}