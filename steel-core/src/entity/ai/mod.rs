//! Mob AI system with goals and target selection.
//!
//! This module implements vanilla's AI goal system for mobs, including:
//! - GoalSelector for managing active AI goals
//! - TargetSelector for attack target selection
//! - Pathfinding navigation
//! - Common AI behaviors (look at player, random stroll, etc.)

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use steel_utils::locks::SyncMutex;

use crate::entity::LivingEntity;
use crate::world::World;

pub mod goal;
pub mod mobs;
pub mod navigation;
pub mod target;

pub use goal::{Goal, GoalSelector, MoveControl, RandomStrollGoal, Target, TargetSelector};
pub use navigation::Navigation;
pub use target::{HurtByTargetGoal, NearestAttackableTargetGoal};

const DEFAULT_PRIORITY: i32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIStatus {
    Stop,
    Start,
    Continue,
}

pub struct MobAI {
    goal_selector: SyncMutex<GoalSelector>,
    target_selector: SyncMutex<TargetSelector>,
    navigation: SyncMutex<Option<Navigation>>,
    move_control: SyncMutex<MoveControl>,
    last_stroll: AtomicI32,
    speed_mod: f32,
    xxa: f32,
    yya: f32,
    zza: f32,
}

impl MobAI {
    pub fn new() -> Self {
        Self {
            goal_selector: SyncMutex::new(GoalSelector::new()),
            target_selector: SyncMutex::new(TargetSelector::new()),
            navigation: SyncMutex::new(None),
            move_control: SyncMutex::new(MoveControl::new()),
            last_stroll: AtomicI32::new(0),
            speed_mod: 1.0,
            xxa: 0.0,
            yya: 0.0,
            zza: 0.0,
        }
    }

    pub fn init(&self, entity: Arc<dyn LivingEntity>, world: Weak<World>) {
        let nav = Navigation::new(entity, world);
        *self.navigation.lock() = Some(nav);
    }

    pub fn tick(&self, entity: &dyn LivingEntity) {
        {
            let mut goals = self.goal_selector.lock();
            goals.tick(entity);
        }
        {
            let mut targets = self.target_selector.lock();
            targets.tick(entity);
        }

        if let Some(nav) = self.navigation.lock().as_ref() {
            nav.tick();
        }

        let control = self.move_control.lock();
        let speed_mod = self.speed_mod;
        match control.status {
            AIStatus::Start | AIStatus::Continue => {
                let speed = entity.get_speed() * speed_mod;
                let move_dir = DVec3::new(control.x, control.y, control.z);

                if move_dir != DVec3::ZERO {
                    let move_len = move_dir.length();
                    if move_len > 0.0 {
                        let normalized = move_dir / move_len;
                        let velocity = normalized * f64::from(speed);

                        let rot = entity.rotation();
                        let yaw = f64::from(rot.0).to_radians();
                        let (sin, cos) = (yaw.sin(), yaw.cos());

                        let dx = normalized.x * cos - normalized.z * sin;
                        let dz = normalized.x * sin + normalized.z * cos;

                        entity.set_velocity(DVec3::new(
                            velocity.x,
                            entity.velocity().y,
                            velocity.z,
                        ));
                    }
                } else {
                    let vel = entity.velocity();
                    let new_vel = DVec3::new(0.0, vel.y, 0.0);
                    entity.set_velocity(new_vel);
                }
            }
            AIStatus::Stop => {
                let vel = entity.velocity();
                entity.set_velocity(DVec3::new(0.0, vel.y, 0.0));
            }
        }
    }

    pub fn goal_selector(&self) -> &SyncMutex<GoalSelector> {
        &self.goal_selector
    }

    pub fn target_selector(&self) -> &SyncMutex<TargetSelector> {
        &self.target_selector
    }

    pub fn navigation(&self) -> &SyncMutex<Option<Navigation>> {
        &self.navigation
    }
}

impl Default for MobAI {
    fn default() -> Self {
        Self::new()
    }
}