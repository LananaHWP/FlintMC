//! Hostile mob AI goals.

use std::sync::Arc;

use glam::DVec3;

use crate::entity::{LivingEntity, SharedEntity};
use crate::world::World;

use super::ai::{Goal, GoalSelector, Target, TargetSelector};
use super::ai::target::NearestAttackableTargetGoal;

pub struct AttackGoal {
    speed_multiplier: f32,
    check_interval: i32,
    target_ticks: i32,
}

impl AttackGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            speed_multiplier: speed,
            check_interval: 20,
            target_ticks: 0,
        }
    }
}

impl Goal for AttackGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if self.target_ticks > 0 {
            self.target_ticks -= 1;
            return;
        }

        if self.target_ticks <= 0 {
            self.target_ticks = self.check_interval;
        }
    }

    fn can_use(&self, entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        1
    }
}

pub struct ZombieBreakDoorGoal {
    breaking: bool,
    door_position: Option<DVec3>,
}

impl ZombieBreakDoorGoal {
    pub fn new() -> Self {
        Self {
            breaking: false,
            door_position: None,
        }
    }
}

impl Goal for ZombieBreakDoorGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        self.breaking
    }

    fn get_priority(&self) -> i32 {
        3
    }
}

impl Default for ZombieBreakDoorGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_hostile_ai(goals: &mut GoalSelector, targets: &mut TargetSelector) {
    let attack_goal = Box::new(AttackGoal::new(1.0));
    goals.add_goal(attack_goal, 1);

    let nearest_target = Box::new(NearestAttackableTargetGoal::new());
    targets.add_target(nearest_target, 1);
}