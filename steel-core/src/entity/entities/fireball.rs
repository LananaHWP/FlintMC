//! Fireball entity implementations.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use steel_registry::blocks::shapes::AABBd;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::entity_types::EntityTypeRef;
use steel_registry::vanilla_entities;
use steel_utils::locks::SyncMutex;
use uuid::Uuid;

use crate::entity::damage::DamageSource;
use crate::entity::{Entity, EntityBase, RemovalReason, SharedEntity};
use crate::world::World;
use steel_registry::vanilla_damage_types;

const DEFAULT_GRAVITY: f64 = 0.05;
const DRAG: f64 = 0.99;
const MAX_LIFETIME: i32 = 1200;
const EXPLOSION_RADIUS: f32 = 1.0;

const VELOCITY_SYNC_THRESHOLD: f64 = 1.0e-7;
const POSITION_SYNC_THRESHOLD: f64 = 7.629_394_5e-6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FireballState {
    Flying,
    Exploded,
    Removed,
}

pub struct SmallFireballEntity {
    base: EntityBase,
    velocity: SyncMutex<DVec3>,
    owner: SyncMutex<Option<Uuid>>,
    state: AtomicI32,
    lifetime: AtomicI32,
    damage: SyncMutex<f32>,
    explosion_radius: SyncMutex<f32>,
    last_sent_velocity: SyncMutex<DVec3>,
    last_sent_position: SyncMutex<DVec3>,
    needs_sync: AtomicBool,
    tick_count: AtomicI32,
}

impl SmallFireballEntity {
    #[must_use]
    pub fn new(id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::with_velocity(id, position, DVec3::new(0.0, 0.0, 0.0), world)
    }

    #[must_use]
    pub fn with_velocity(
        id: i32,
        position: DVec3,
        velocity: DVec3,
        world: Weak<World>,
    ) -> Self {
        Self {
            base: EntityBase::new(id, position, world),
            velocity: SyncMutex::new(velocity),
            owner: SyncMutex::new(None),
            state: AtomicI32::new(FireballState::Flying as i32),
            lifetime: AtomicI32::new(0),
            damage: SyncMutex::new(5.0),
            explosion_radius: SyncMutex::new(EXPLOSION_RADIUS),
            last_sent_velocity: SyncMutex::new(velocity),
            last_sent_position: SyncMutex::new(position),
            needs_sync: AtomicBool::new(false),
            tick_count: AtomicI32::new(0),
        }
    }

    pub fn set_owner(&self, owner: Uuid) {
        *self.owner.lock() = Some(owner);
    }

    #[must_use]
    pub fn get_owner(&self) -> Option<Uuid> {
        *self.owner.lock()
    }

    #[must_use]
    pub fn get_state(&self) -> FireballState {
        match self.state.load(Ordering::Relaxed) {
            0 => FireballState::Flying,
            1 => FireballState::Exploded,
            _ => FireballState::Removed,
        }
    }

    fn set_state(&self, state: FireballState) {
        self.state.store(state as i32, Ordering::Relaxed);
    }

    fn get_current_tick(&self) -> i32 {
        self.tick_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn tick_flying(&self) {
        let world = match self.level() {
            Some(w) => w,
            None => {
                self.set_removed(RemovalReason::Discarded);
                return;
            }
        };

        let pos = self.position();

        let mut vel = self.velocity();
        vel.y -= DEFAULT_GRAVITY;
        vel.x *= DRAG;
        vel.z *= DRAG;
        *self.velocity.lock() = vel;

        let new_pos = pos + vel;
        self.set_position(new_pos);

        let current_tick = self.get_current_tick();

        if current_tick % 4 == 0 || vel.length() > 0.0 {
            let block_pos = steel_utils::BlockPos::new(
                new_pos.x.floor() as i32,
                new_pos.y.floor() as i32,
                new_pos.z.floor() as i32,
            );
            let block_state = world.get_block_state(block_pos);
            if !block_state.is_air() {
                self.explode(&world);
                return;
            }
        }

        if current_tick > MAX_LIFETIME {
            self.set_removed(RemovalReason::Discarded);
            return;
        }

        self.check_entity_collision(&world);
    }

    fn check_entity_collision(&self, world: &Arc<World>) {
        let pos = self.position();
        let aabb = AABBd {
            min_x: pos.x - 0.5,
            min_y: pos.y - 0.5,
            min_z: pos.z - 0.5,
            max_x: pos.x + 0.5,
            max_y: pos.y + 0.5,
            max_z: pos.z + 0.5,
        };

        for entity in world.get_entities_in_aabb(&aabb) {
            if entity.id() == self.id() {
                continue;
            }

            let entity_pos = entity.position();
            let dist_sq = (pos.x - entity_pos.x).powi(2)
                + (pos.y - entity_pos.y).powi(2)
                + (pos.z - entity_pos.z).powi(2);

            if dist_sq < 0.5 {
                self.hit_entity(entity);
                return;
            }
        }
    }

    fn hit_entity(&self, target: SharedEntity) {
        let damage = *self.damage.lock();

        let source = DamageSource::entity(&vanilla_damage_types::ON_FIRE, target.id());
        target.hurt(&source, damage);

        if let Some(world) = target.level() {
            self.explode(&world);
        }
    }

    fn explode(&self, world: &Arc<World>) {
        self.set_state(FireballState::Exploded);

        let explosion_radius = *self.explosion_radius.lock();

        let pos = self.position();
        let block_pos = steel_utils::BlockPos::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );

        let block_state = world.get_block_state(block_pos);
        if block_state.is_solid() {
            let damage = *self.damage.lock();
            let aabb = AABBd {
                min_x: pos.x - 1.0,
                min_y: pos.y - 1.0,
                min_z: pos.z - 1.0,
                max_x: pos.x + 1.0,
                max_y: pos.y + 1.0,
                max_z: pos.z + 1.0,
            };

            for entity in world.get_entities_in_aabb(&aabb) {
                let entity_pos = entity.position();
                let dist = ((pos.x - entity_pos.x).powi(2)
                    + (pos.y - entity_pos.y).powi(2)
                    + (pos.z - entity_pos.z).powi(2))
                .sqrt();

                if dist < f64::from(explosion_radius) {
                    let entity_damage = damage * (1.0 - (dist / f64::from(explosion_radius)) as f32);
                    let source = DamageSource::entity(&vanilla_damage_types::ON_FIRE, self.id());
                    entity.hurt(&source, entity_damage as f32);
                }
            }
        }

        self.set_removed(RemovalReason::Discarded);
    }
}

impl Entity for SmallFireballEntity {
    fn base(&self) -> Option<&EntityBase> {
        Some(&self.base)
    }

    fn entity_type(&self) -> EntityTypeRef {
        &vanilla_entities::SMALL_FIREBALL
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
        self.lifetime.fetch_add(1, Ordering::Relaxed);

        match self.get_state() {
            FireballState::Flying => self.tick_flying(),
            FireballState::Exploded => {}
            FireballState::Removed => {}
        }
    }

    fn send_changes(&self, tick_count: i32) {
        let Some(world) = self.level() else {
            return;
        };

        let update_interval = self.entity_type().update_interval;
        let needs_sync = self.needs_sync.load(Ordering::Relaxed);

        if tick_count % update_interval != 0 && !needs_sync {
            return;
        }

        let current_pos = self.position();
        let chunk_pos = steel_utils::ChunkPos::new(
            (current_pos.x as i32) >> 4,
            (current_pos.z as i32) >> 4,
        );

        let vel = self.velocity();
        let diff_sq = (vel.x - self.last_sent_velocity.lock().x).powi(2)
            + (vel.y - self.last_sent_velocity.lock().y).powi(2)
            + (vel.z - self.last_sent_velocity.lock().z).powi(2);

        if diff_sq > VELOCITY_SYNC_THRESHOLD
            || (diff_sq > 0.0
                && vel.x == 0.0
                && vel.y == 0.0
                && vel.z == 0.0)
        {
            use steel_protocol::packets::game::CSetEntityMotion;
            let packet = CSetEntityMotion::new(self.id(), vel.x, vel.y, vel.z);
            *self.last_sent_velocity.lock() = vel;
            world.broadcast_to_nearby(chunk_pos, packet, None);
        }

        let diff_pos_sq = (current_pos.x - self.last_sent_position.lock().x).powi(2)
            + (current_pos.y - self.last_sent_position.lock().y).powi(2)
            + (current_pos.z - self.last_sent_position.lock().z).powi(2);

        if diff_pos_sq >= POSITION_SYNC_THRESHOLD
            || needs_sync
            || tick_count % 60 == 0
        {
            use steel_protocol::packets::game::CEntityPositionSync;
            let packet = CEntityPositionSync {
                entity_id: self.id(),
                x: current_pos.x,
                y: current_pos.y,
                z: current_pos.z,
                velocity_x: vel.x,
                velocity_y: vel.y,
                velocity_z: vel.z,
                yaw: 0.0,
                pitch: 0.0,
                on_ground: false,
            };
            *self.last_sent_position.lock() = current_pos;
            world.broadcast_to_nearby(chunk_pos, packet, None);
        }

        if needs_sync {
            self.needs_sync.store(false, Ordering::Relaxed);
        }
    }

    fn get_default_gravity(&self) -> f64 {
        DEFAULT_GRAVITY
    }

    fn velocity(&self) -> DVec3 {
        *self.velocity.lock()
    }

    fn set_velocity(&self, velocity: DVec3) {
        *self.velocity.lock() = velocity;
    }
}