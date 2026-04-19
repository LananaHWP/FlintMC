//! AI Goal system for mobs.
//!
//! Implements vanilla's Goal system - a behavior tree where goals can be
//! prioritized and executed based on conditions.

use std::sync::atomic::AtomicI32;

use crate::entity::{LivingEntity, SharedEntity};

pub mod behaviors;

pub use behaviors::*;

const DEFAULT_PRIORITY: i32 = 0;

pub trait Goal: Send + Sync {
    fn tick(&mut self, entity: &dyn LivingEntity);

    fn can_use(&self, entity: &dyn LivingEntity) -> bool;

    fn can_continue_to_use(&self, entity: &dyn LivingEntity) -> bool {
        self.can_use(entity)
    }

    fn is_interruptable(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        DEFAULT_PRIORITY
    }

    fn start(&mut self, _entity: &dyn LivingEntity) {}

    fn stop(&mut self, _entity: &dyn LivingEntity) {}
}

pub struct GoalSelector {
    available_goals: Vec<(Box<dyn Goal>, i32)>,
    active_goals: Vec<usize>,
    tick_count: AtomicI32,
    was_active: Vec<bool>,
}

impl GoalSelector {
    pub fn new() -> Self {
        Self {
            available_goals: Vec::new(),
            active_goals: Vec::new(),
            tick_count: AtomicI32::new(0),
            was_active: Vec::new(),
        }
    }

    pub fn add_goal(&mut self, goal: Box<dyn Goal>, priority: i32) {
        self.available_goals.push((goal, priority));
        self.available_goals.sort_by(|a, b| b.1.cmp(&a.1));
    }

    pub fn tick(&mut self, entity: &dyn LivingEntity) {
        self.tick_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut new_active = Vec::new();

        for &(ref goal, priority) in &self.available_goals {
            if goal.can_use(entity) {
                for (i, &(ref g, p)) in self.available_goals.iter().enumerate() {
                    if p == priority && g.can_use(entity) {
                        new_active.push(i);
                    }
                }
                break;
            }
        }

        for &i in &new_active {
            if !self.was_active.get(i).map_or(false, |w| *w) {
                if let Some(ref mut goal) = self.available_goals.get_mut(i) {
                    goal.0.start(entity);
                }
            }
        }

        for (i, was) in self.was_active.iter_mut().enumerate() {
            if *was && !new_active.contains(&i) {
                if let Some(ref mut goal) = self.available_goals.get_mut(i) {
                    goal.0.stop(entity);
                }
            }
        }

        while self.was_active.len() < self.available_goals.len() {
            self.was_active.push(false);
        }
        for i in 0..self.was_active.len() {
            self.was_active[i] = new_active.contains(&i);
        }

        self.active_goals = new_active;

        for &i in &self.active_goals {
            if let Some(ref mut goal) = self.available_goals.get_mut(i) {
                goal.0.tick(entity);
            }
        }
    }

    pub fn remove_goal(&mut self, _goal: &dyn Goal) {}

    pub fn remove_all_goals(&mut self) {
        self.available_goals.clear();
        self.active_goals.clear();
    }
}

impl Default for GoalSelector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MoveControl {
    pub status: super::AIStatus,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub jump: bool,
}

impl MoveControl {
    pub fn new() -> Self {
        Self {
            status: super::AIStatus::Stop,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            jump: false,
        }
    }

    pub fn set_move_direction(&mut self, x: f64, y: f64, z: f64) {
        self.x = x;
        self.y = y;
        self.z = z;
        if x != 0.0 || y != 0.0 || z != 0.0 {
            self.status = super::AIStatus::Start;
        } else {
            self.status = super::AIStatus::Stop;
        }
    }

    pub fn set_jump(&mut self, jump: bool) {
        self.jump = jump;
    }
}

impl Default for MoveControl {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Target: Send + Sync {
    fn tick(&mut self, entity: &dyn LivingEntity);

    fn can_use(&self, entity: &dyn LivingEntity) -> bool;

    fn get_priority(&self) -> i32 {
        DEFAULT_PRIORITY
    }

    fn set_target(&mut self, entity: &dyn LivingEntity, target: Option<SharedEntity>);

    fn get_target(&self) -> Option<SharedEntity>;
}

pub struct TargetSelector {
    available_targets: Vec<(Box<dyn Target>, i32)>,
    current_target: Option<SharedEntity>,
    targeting_entity: Option<SharedEntity>,
    using_secondary: bool,
    tick_count: i32,
}

impl TargetSelector {
    pub fn new() -> Self {
        Self {
            available_targets: Vec::new(),
            current_target: None,
            targeting_entity: None,
            using_secondary: false,
            tick_count: 0,
        }
    }

    pub fn add_target(&mut self, target: Box<dyn Target>, priority: i32) {
        self.available_targets.push((target, priority));
        self.available_targets.sort_by(|a, b| b.1.cmp(&a.1));
    }

    pub fn tick(&mut self, entity: &dyn LivingEntity) {
        self.tick_count += 1;

        if self.tick_count % 20 == 0 {
            for (target_goal, _) in &mut self.available_targets {
                if target_goal.can_use(entity) {
                    self.current_target = target_goal.get_target();
                    break;
                }
            }
        }

        for (target_goal, _) in &mut self.available_targets {
            target_goal.tick(entity);
        }
    }

    pub fn set_target(&mut self, target: Option<SharedEntity>) {
        self.current_target = target;
    }

    pub fn get_target(&self) -> Option<SharedEntity> {
        self.current_target.clone()
    }

    pub fn clear_targets(&mut self) {
        self.available_targets.clear();
        self.current_target = None;
    }
}

impl Default for TargetSelector {
    fn default() -> Self {
        Self::new()
    }
}