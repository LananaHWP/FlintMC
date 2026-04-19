//! Target selector goals for mob AI.

use std::sync::Arc;

use crate::entity::{LivingEntity, SharedEntity};
use crate::world::World;

use super::Target;

const DEFAULT_PRIORITY: i32 = 0;

pub struct NearestAttackableTargetGoal {
    target: Option<SharedEntity>,
    target_ticks: i32,
    target_interval: i32,
    seeing_target: bool,
    closest_target: bool,
    target_distance: f64,
}

impl NearestAttackableTargetGoal {
    pub fn new() -> Self {
        Self {
            target: None,
            target_ticks: 0,
            target_interval: 0,
            seeing_target: false,
            closest_target: false,
            target_distance: 16.0,
        }
    }

    pub fn set_target_distance(&mut self, distance: f64) {
        self.target_distance = distance;
    }
}

impl Target for NearestAttackableTargetGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        DEFAULT_PRIORITY
    }

    fn set_target(&mut self, _entity: &dyn LivingEntity, target: Option<SharedEntity>) {
        self.target = target;
    }

    fn get_target(&self) -> Option<SharedEntity> {
        self.target.clone()
    }
}

impl Default for NearestAttackableTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HurtByTargetGoal {
    target: Option<SharedEntity>,
    last_hurt_by: Option<SharedEntity>,
}

impl HurtByTargetGoal {
    pub fn new() -> Self {
        Self {
            target: None,
            last_hurt_by: None,
        }
    }
}

impl Target for HurtByTargetGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        1
    }

    fn set_target(&mut self, _entity: &dyn LivingEntity, target: Option<SharedEntity>) {
        self.target = target;
    }

    fn get_target(&self) -> Option<SharedEntity> {
        self.target.clone()
    }
}

impl Default for HurtByTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}