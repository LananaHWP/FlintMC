//! Common AI goal behaviors for mobs.

use std::sync::atomic::Ordering;

use glam::DVec3;
use steel_utils::BlockPos;

use crate::entity::damage::DamageSource;
use crate::entity::{LivingEntity, SharedEntity};
use crate::world::World;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::vanilla_attributes;

use super::Goal;

pub struct MeleeAttackGoal {
    speed_multiplier: f32,
    check_interval: i32,
    target_ticks: i32,
    min_attack_distance: f64,
    target: Option<SharedEntity>,
}

impl MeleeAttackGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            speed_multiplier: speed,
            check_interval: 20,
            target_ticks: 0,
            min_attack_distance: f64::MAX,
            target: None,
        }
    }

    pub fn new_with_min_distance(speed: f32, min_distance: f64) -> Self {
        Self {
            speed_multiplier: speed,
            check_interval: 20,
            target_ticks: 0,
            min_attack_distance: min_distance,
            target: None,
        }
    }

    pub fn set_target(&mut self, target: Option<SharedEntity>) {
        self.target = target;
    }

    pub fn get_target(&self) -> Option<SharedEntity> {
        self.target.clone()
    }
}

impl Goal for MeleeAttackGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if let Some(ref target) = self.target {
            let mob_pos = entity.position();
            let target_pos = target.position();
            let dist = mob_pos.distance(target_pos);

            if dist < self.min_attack_distance {
                let attack_damage = entity
                    .attributes()
                    .lock()
                    .get_value(vanilla_attributes::ATTACK_DAMAGE)
                    .unwrap_or(1.0) as f32;
                
                target.hurt(&DamageSource::generic_mob_attack(), attack_damage);
            } else {
                let speed = entity.get_speed() * self.speed_multiplier;
                let direction = target_pos - mob_pos;
                let dist = direction.length();
                if dist > 0.1 {
                    let dx = direction.x / dist * f64::from(speed);
                    let dz = direction.z / dist * f64::from(speed);

                    let current_vel = entity.velocity();
                    entity.set_velocity(DVec3::new(dx, current_vel.y, dz));
                }
            }
        }
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        self.target.is_some()
    }

    fn get_priority(&self) -> i32 {
        1
    }

    fn start(&mut self, _entity: &dyn LivingEntity) {
        self.target_ticks = 0;
    }

    fn stop(&mut self, _entity: &dyn LivingEntity) {
        self.target = None;
    }
}

pub struct RangedAttackGoal {
    attack_interval: i32,
    attack_time: i32,
}

impl RangedAttackGoal {
    pub fn new(attack_interval: i32) -> Self {
        Self {
            attack_interval,
            attack_time: 0,
        }
    }
}

impl Goal for RangedAttackGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        1
    }
}

pub struct LeapAtTargetGoal {
    chance: f32,
    y_height: f64,
}

impl LeapAtTargetGoal {
    pub fn new(chance: f32, y_height: f64) -> Self {
        Self { chance, y_height }
    }
}

impl Goal for LeapAtTargetGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }
}

pub struct HurtByTargetGoal {
    target: Option<crate::entity::SharedEntity>,
}

impl HurtByTargetGoal {
    pub fn new() -> Self {
        Self { target: None }
    }
}

impl Goal for HurtByTargetGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        1
    }
}

impl Default for HurtByTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FindEntityGoal {
    radius: f64,
    entity_type: Option<String>,
}

impl FindEntityGoal {
    pub fn new(radius: f64) -> Self {
        Self {
            radius,
            entity_type: None,
        }
    }

    pub fn with_entity_type(mut self, entity_type: &str) -> Self {
        self.entity_type = Some(entity_type.to_string());
        self
    }
}

impl Goal for FindEntityGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        5
    }
}

pub struct TemptGoal {
    speed_multiplier: f32,
    scared_by_nearby_player: bool,
}

impl TemptGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            speed_multiplier: speed,
            scared_by_nearby_player: false,
        }
    }
}

impl Goal for TemptGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if let Some(world) = entity.level() {
            let mob_pos = entity.position();
            let block_pos = BlockPos::new(mob_pos.x as i32, mob_pos.y as i32, mob_pos.z as i32);

            let nearby_entities = world.get_nearby_entities(block_pos, 10);
            
            let mut closest_player: Option<SharedEntity> = None;
            let mut closest_dist = f64::MAX;

            for e in nearby_entities {
                if e.clone().as_player().is_some() {
                let _player = e.clone().as_player();
                    let dist = mob_pos.distance(e.position());
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_player = Some(e);
                    }
                }
            }

            if let Some(player) = closest_player {
                let player_pos = player.position();
                let speed = entity.get_speed() * self.speed_multiplier;
                let direction = player_pos - mob_pos;
                let dist = direction.length();

                if dist > 1.5 {
                    let dx = direction.x / dist * f64::from(speed);
                    let dz = direction.z / dist * f64::from(speed);

                    let current_vel = entity.velocity();
                    entity.set_velocity(DVec3::new(dx, current_vel.y, dz));
                }
            }
        }
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        3
    }
}

pub struct FollowParentGoal {
    speed_multiplier: f32,
    parent_distance: f64,
    start_distance: f64,
}

impl FollowParentGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            speed_multiplier: speed,
            parent_distance: 4.0,
            start_distance: 9.0,
        }
    }
}

impl Goal for FollowParentGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        5
    }
}

pub struct BreedGoal {
    speed_multiplier: f32,
    partner_distance: f64,
}

impl BreedGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            speed_multiplier: speed,
            partner_distance: 9.0,
        }
    }
}

impl Goal for BreedGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        3
    }
}

pub struct AvoidEntityGoal {
    flee_distance: f64,
    far_distance: f64,
    near_mob: bool,
}

impl AvoidEntityGoal {
    pub fn new(flee_distance: f64) -> Self {
        Self {
            flee_distance,
            far_distance: flee_distance * 2.0,
            near_mob: false,
        }
    }
}

impl Goal for AvoidEntityGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        3
    }
}

pub struct SitGoal {
    sitting: bool,
}

impl SitGoal {
    pub fn new() -> Self {
        Self { sitting: false }
    }
}

impl Goal for SitGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }
}

impl Default for SitGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OwnerHurtByTargetGoal {
    owner: Option<crate::entity::SharedEntity>,
    target: Option<(crate::entity::SharedEntity, i32)>,
}

impl OwnerHurtByTargetGoal {
    pub fn new() -> Self {
        Self {
            owner: None,
            target: None,
        }
    }
}

impl Goal for OwnerHurtByTargetGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }
}

impl Default for OwnerHurtByTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OwnerTargetGoal {
    target: Option<crate::entity::SharedEntity>,
}

impl OwnerTargetGoal {
    pub fn new() -> Self {
        Self { target: None }
    }
}

impl Goal for OwnerTargetGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }
}

impl Default for OwnerTargetGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JumpWithOwnerGoal {
    jump_chance: f32,
}

impl JumpWithOwnerGoal {
    pub fn new() -> Self {
        Self { jump_chance: f32::MAX }
    }
}

impl Goal for JumpWithOwnerGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        5
    }
}

impl Default for JumpWithOwnerGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RandomLookaroundGoal {
    look_time: i32,
    interval: i32,
}

impl RandomLookaroundGoal {
    pub fn new() -> Self {
        Self {
            look_time: 0,
            interval: 20,
        }
    }
}

impl Goal for RandomLookaroundGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        -1
    }
}

impl Default for RandomLookaroundGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ClimbGoal {}

impl ClimbGoal {
    pub fn new() -> Self {
        Self {}
    }
}

impl Goal for ClimbGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if let Some(world) = entity.level() {
            let pos = entity.position();
            let block_pos = BlockPos::new(pos.x as i32, pos.y as i32, pos.z as i32);

            let north_pos = BlockPos::new(block_pos.x(), block_pos.y(), block_pos.z() - 1);
            let south_pos = BlockPos::new(block_pos.x(), block_pos.y(), block_pos.z() + 1);
            let east_pos = BlockPos::new(block_pos.x() + 1, block_pos.y(), block_pos.z());
            let west_pos = BlockPos::new(block_pos.x() - 1, block_pos.y(), block_pos.z());

            let world_ref = world.clone();
            let is_climbable = |pos: BlockPos| -> bool {
                let state = world_ref.get_block_state(pos);
                let block = state.get_block();
                block.key.path.contains("fence") || 
                block.key.path.contains("wall") ||
                block.key.path.contains(" vines") ||
                block.key.path.contains("ladder") ||
                block.key.path.contains("iron_bars")
            };

            let can_climb = is_climbable(north_pos) || is_climbable(south_pos) || 
                         is_climbable(east_pos) || is_climbable(west_pos);

            if can_climb {
                entity.set_y_velocity(0.2);
            }
        }
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        1
    }
}

impl Default for ClimbGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TeleportWhenTargetGoneGoal {
    target_change_interval: i32,
    too_far_distance: f64,
    teleport_timer: i32,
}

impl TeleportWhenTargetGoneGoal {
    pub fn new() -> Self {
        Self {
            target_change_interval: 0,
            too_far_distance: 16.0,
            teleport_timer: 0,
        }
    }
}

impl Goal for TeleportWhenTargetGoneGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if let Some(world) = entity.level() {
            let mob_pos = entity.position();
            let block_pos = BlockPos::new(mob_pos.x as i32, mob_pos.y as i32, mob_pos.z as i32);

            let state = world.get_block_state(block_pos);
            let block = state.get_block();
            let is_solid = state.is_solid();

            let state_above = world.get_block_state(BlockPos::new(
                mob_pos.x as i32, 
                (mob_pos.y + 1.0) as i32, 
                mob_pos.z as i32
            ));
            let above_solid = state_above.is_solid();

            if !is_solid || !above_solid {
                self.teleport_timer += 1;
                if self.teleport_timer > 200 {
                    self.try_teleport(entity, &world);
                    self.teleport_timer = 0;
                }
            } else {
                self.teleport_timer = 0;
            }
        }
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }
}

impl TeleportWhenTargetGoneGoal {
    fn try_teleport(&mut self, entity: &dyn LivingEntity, world: &World) {
        let mob_pos = entity.position();
        let r = rand::random::<f64>() * 8.0 - 4.0;
        let angle = rand::random::<f64>() * std::f64::consts::TAU;
        
        let new_x = (mob_pos.x + r * angle.cos()) as i32;
        let new_z = (mob_pos.z + r * angle.sin()) as i32;
        
        let search_start_y = mob_pos.y as i32;
        
        for y_offset in 0..=10_i32 {
            let new_y = search_start_y - y_offset;
            if new_y < -64 {
                break;
            }
            
            let pos = BlockPos::new(new_x, new_y, new_z);
            let state = world.get_block_state(pos);
            let state_above = world.get_block_state(BlockPos::new(new_x, new_y + 1, new_z));
            
            let is_air = !state.is_solid() && state.get_block().key.path == "minecraft:air";
            let above_solid = state_above.is_solid();
            
            if is_air && above_solid {
                let new_position = DVec3::new(
                    f64::from(new_x) + 0.5,
                    f64::from(new_y),
                    f64::from(new_z) + 0.5,
                );
                entity.set_position(new_position);
                break;
            }
        }
    }
}

impl Default for TeleportWhenTargetGoneGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MoveBackToYGoal {
    water_adjustment: f64,
    speed_multiplier: f32,
    check_interval: i32,
}

impl MoveBackToYGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            water_adjustment: 0.0,
            speed_multiplier: speed,
            check_interval: 0,
        }
    }
}

impl Goal for MoveBackToYGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }
}

pub struct LookAtPlayerGoal {
    see_speed: f64,
    xxhold: f64,
    look_distance: f64,
}

impl LookAtPlayerGoal {
    pub fn new() -> Self {
        Self {
            see_speed: 0.02,
            xxhold: 0.0,
            look_distance: 8.0,
        }
    }
}

impl Goal for LookAtPlayerGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        5
    }
}

impl Default for LookAtPlayerGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RandomStrollGoal {
    speed_multiplier: f32,
    xa: f64,
    ya: f64,
    za: f64,
    tick_count: i32,
    next_tick: i32,
}

impl RandomStrollGoal {
    pub fn new(speed: f32) -> Self {
        Self {
            speed_multiplier: speed,
            xa: 0.0,
            ya: 0.0,
            za: 0.0,
            tick_count: 0,
            next_tick: 0,
        }
    }
}

impl Goal for RandomStrollGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if self.tick_count >= self.next_tick {
            self.tick_count = 0;
            let angle = rand::random::<f64>() * std::f64::consts::TAU;
            self.xa = angle.cos() * self.speed_multiplier as f64;
            self.za = angle.sin() * self.speed_multiplier as f64;
            self.next_tick = (20 + rand::random::<i32>() % 60) as i32;
        }
        self.tick_count += 1;

        if self.xa != 0.0 || self.za != 0.0 {
            let speed = entity.get_speed() * self.speed_multiplier;
            let rot = entity.rotation();
            let yaw = f64::from(rot.0).to_radians();
            let (sin, cos) = (yaw.sin(), yaw.cos());

            let dx = self.xa * cos as f64 - self.za * sin as f64;
            let dz = self.xa * sin as f64 + self.za * cos as f64;

            let current_vel = entity.velocity();
            entity.set_velocity(DVec3::new(dx * f64::from(speed), current_vel.y, dz * f64::from(speed)));
        }
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }

    fn start(&mut self, _entity: &dyn LivingEntity) {
        self.tick_count = 0;
        self.next_tick = 0;
        self.xa = 0.0;
        self.za = 0.0;
    }

    fn stop(&mut self, entity: &dyn LivingEntity) {
        self.xa = 0.0;
        self.za = 0.0;
        entity.set_velocity(DVec3::new(0.0, entity.velocity().y, 0.0));
    }
}

pub struct FloatGoal {
    jump_chance: f32,
}

impl FloatGoal {
    pub fn new(jump_chance: f32) -> Self {
        Self { jump_chance }
    }
}

impl Goal for FloatGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        if let Some(world) = entity.level() {
            let pos = entity.position();
            let block_pos = BlockPos::new(pos.x as i32, pos.y as i32, pos.z as i32);
            
            let state = world.get_block_state(block_pos);
            let is_water = state.is_water();
            
            let pos_above = BlockPos::new(pos.x as i32, (pos.y + 1.0) as i32, pos.z as i32);
            let state_above = world.get_block_state(pos_above);
            let above_water = state_above.is_water();

            if is_water || above_water {
                if rand::random::<f32>() < self.jump_chance {
                    entity.set_y_velocity(0.1);
                }
            }
        }
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        false
    }

    fn get_priority(&self) -> i32 {
        1
    }
}

pub struct BreakDoorGoal {
    breaking: std::sync::atomic::AtomicBool,
}

impl BreakDoorGoal {
    pub fn new() -> Self {
        Self {
            breaking: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Goal for BreakDoorGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        self.breaking.load(Ordering::Relaxed)
    }

    fn get_priority(&self) -> i32 {
        3
    }

    fn start(&mut self, _entity: &dyn LivingEntity) {
        self.breaking.store(true, Ordering::Relaxed);
    }

    fn stop(&mut self, _entity: &dyn LivingEntity) {
        self.breaking.store(false, Ordering::Relaxed);
    }
}

impl Default for BreakDoorGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OpenDoorGoal {
    ignore_door: bool,
}

impl OpenDoorGoal {
    pub fn new() -> Self {
        Self { ignore_door: false }
    }

    pub fn with_ignore_door(mut self, ignore: bool) -> Self {
        self.ignore_door = ignore;
        self
    }
}

impl Goal for OpenDoorGoal {
    fn tick(&mut self, _entity: &dyn LivingEntity) {}

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        if self.ignore_door {
            99
        } else {
            4
        }
    }
}

impl Default for OpenDoorGoal {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WaterAvoidingRandomStrollGoal {
    speed_multiplier: f32,
    radius: i32,
    xa: f64,
    ya: f64,
    za: f64,
    tick_count: i32,
}

impl WaterAvoidingRandomStrollGoal {
    pub fn new(speed: f32, radius: i32) -> Self {
        Self {
            speed_multiplier: speed,
            radius,
            xa: 0.0,
            ya: 0.0,
            za: 0.0,
            tick_count: 0,
        }
    }

    fn generate_new_wander(&mut self) {
        let angle = rand::random::<f64>() * std::f64::consts::TAU;
        let dist = rand::random::<f64>() * self.radius as f64;

        self.xa = angle.cos() * dist;
        self.za = angle.sin() * dist;
        self.ya = 0.0;

        self.tick_count = 0;
    }
}

impl Goal for WaterAvoidingRandomStrollGoal {
    fn tick(&mut self, entity: &dyn LivingEntity) {
        let pos = entity.position();
        if pos.y >= 60.0 {
            self.ya = -1.0;
        }

        if self.tick_count == 0 || (self.xa == 0.0 && self.za == 0.0 && self.ya == 0.0) {
            self.generate_new_wander();
        }
        self.tick_count += 1;
    }

    fn can_use(&self, _entity: &dyn LivingEntity) -> bool {
        true
    }

    fn get_priority(&self) -> i32 {
        2
    }

    fn stop(&mut self, _entity: &dyn LivingEntity) {
        self.xa = 0.0;
        self.ya = 0.0;
        self.za = 0.0;
    }
}