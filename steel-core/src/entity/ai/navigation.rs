//! Navigation system with pathfinding for mobs.
//!
//! Implements vanilla's PathNavigation system using A* pathfinding.

use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use glam::{DVec3, IVec3};
use steel_utils::locks::SyncMutex;
use steel_utils::BlockPos;

use crate::entity::LivingEntity;
use crate::world::World;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::Block;
use steel_registry::vanilla_blocks;

pub struct Navigation {
    entity: Arc<dyn LivingEntity>,
    world: Weak<World>,
    path: SyncMutex<Vec<PathNode>>,
    path_index: SyncMutex<usize>,
    target_pos: SyncMutex<Option<DVec3>>,
    can_open_doors: AtomicBool,
    can_pass_doors: AtomicBool,
    can_float: AtomicBool,
    avoid_sun: AtomicBool,
    brute_force: AtomicBool,
    speed: f64,
}

impl Navigation {
    pub fn new(entity: Arc<dyn LivingEntity>, world: Weak<World>) -> Self {
        Self {
            entity,
            world,
            path: SyncMutex::new(Vec::new()),
            path_index: SyncMutex::new(0),
            target_pos: SyncMutex::new(None),
            can_open_doors: AtomicBool::new(false),
            can_pass_doors: AtomicBool::new(false),
            can_float: AtomicBool::new(false),
            avoid_sun: AtomicBool::new(false),
            brute_force: AtomicBool::new(false),
            speed: 0.0,
        }
    }

    pub fn set_can_open_doors(&self, can_open: bool) {
        self.can_open_doors.store(can_open, Ordering::Relaxed);
    }

    pub fn set_can_pass_doors(&self, can_pass: bool) {
        self.can_pass_doors.store(can_pass, Ordering::Relaxed);
    }

    pub fn set_can_float(&self, can_float: bool) {
        self.can_float.store(can_float, Ordering::Relaxed);
    }

    pub fn set_brute_force(&self, brute_force: bool) {
        self.brute_force.store(brute_force, Ordering::Relaxed);
    }

    pub fn set_avoid_sun(&self, avoid: bool) {
        self.avoid_sun.store(avoid, Ordering::Relaxed);
    }

    pub fn set_speed_modifier(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn set_target_position(&self, pos: DVec3) -> bool {
        *self.target_pos.lock() = Some(pos);
        self.recompute_path()
    }

    #[expect(unused_variables)]
    pub fn tick(&self) {
        let world = match self.world.upgrade() {
            Some(w) => w,
            None => return,
        };

        let path = self.path.lock();
        let index = *self.path_index.lock();

        if index < path.len() {
            let node = &path[index];
            let current_pos = self.entity.position();

            let dx = f64::from(node.x) - current_pos.x;
            let dz = f64::from(node.z) - current_pos.z;

            self.entity.set_velocity(DVec3::new(dx * 0.5, current_pos.y, dz * 0.5));

            let dist_sq = dx * dx + dz * dz;
            if dist_sq < 0.5 {
                *self.path_index.lock() = index + 1;
            }
        }
    }

    fn recompute_path(&self) -> bool {
        let start_pos = self.entity.position();
        let target = self.target_pos.lock();

        if target.is_none() {
            return false;
        }

        let target = target.unwrap();

        let start_ivec = IVec3::new(
            start_pos.x as i32,
            start_pos.y as i32,
            start_pos.z as i32,
        );
        let target_ivec = IVec3::new(
            target.x as i32,
            target.y as i32,
            target.z as i32,
        );

        if self.brute_force.load(Ordering::Relaxed) {
            return self.compute_brute_force_path(start_ivec, target_ivec);
        }

        self.compute_path(start_ivec, target_ivec)
    }

    fn compute_brute_force_path(&self, start: IVec3, target: IVec3) -> bool {
        let mut path = Vec::new();
        let mut current = start;

        let world = match self.world.upgrade() {
            Some(w) => w,
            None => return false,
        };

        while current != target {
            let dx = target.x - current.x;
            let dy = target.y - current.y;
            let dz = target.z - current.z;

            let mut next = current;

            if dx != 0 {
                next.x += dx.signum();
            } else if dy != 0 {
                next.y += dy.signum();
            } else if dz != 0 {
                next.z += dz.signum();
            }

            if !self.is_walkable(next, &world) {
                return false;
            }

            path.push(PathNode::new_standalone(next.x, next.y, next.z));
            current = next;
        }

        *self.path.lock() = path;
        *self.path_index.lock() = 0;
        true
    }

    fn compute_path(&self, start: IVec3, target: IVec3) -> bool {
        let world = match self.world.upgrade() {
            Some(w) => w,
            None => return false,
        };

        let mut open_set: BinaryHeap<PathNode> = BinaryHeap::new();
        let mut came_from: HashMap<IVec3, IVec3> = HashMap::new();
        let mut g_score: HashMap<IVec3, f64> = HashMap::new();

        let h = heuristic(start, target);
        let start_node = PathNode::with_f(start.x, start.y, start.z, h);
        g_score.insert(start, 0.0);
        open_set.push(start_node);

        while let Some(current) = open_set.pop() {
            let current_pos = current.as_ivec();

            if current_pos == target {
                let path = self.reconstruct_path(&came_from, current_pos);
                *self.path.lock() = path;
                return true;
            }

            for neighbor in self.get_neighbors(current_pos) {
                if !self.is_walkable(neighbor, &world) {
                    continue;
                }

                let tentative_g = g_score.get(&current_pos).unwrap_or(&f64::MAX) + 1.0;

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&f64::MAX) {
                    came_from.insert(neighbor, current_pos);
                    g_score.insert(neighbor, tentative_g);

                    let h = heuristic(neighbor, target);
                    let f = tentative_g + h;

                    let neighbor_node = PathNode::with_f(neighbor.x, neighbor.y, neighbor.z, f);
                    open_set.push(neighbor_node);
                }
            }
        }

        false
    }

    fn get_neighbors(&self, pos: IVec3) -> Vec<IVec3> {
        vec![
            pos + IVec3::new(1, 0, 0),
            pos + IVec3::new(-1, 0, 0),
            pos + IVec3::new(0, 0, 1),
            pos + IVec3::new(0, 0, -1),
            pos + IVec3::new(1, 1, 0),
            pos + IVec3::new(-1, 1, 0),
            pos + IVec3::new(0, 1, 1),
            pos + IVec3::new(0, 1, -1),
            pos + IVec3::new(1, -1, 0),
            pos + IVec3::new(-1, -1, 0),
            pos + IVec3::new(0, -1, 1),
            pos + IVec3::new(0, -1, -1),
        ]
    }

    fn is_walkable(&self, pos: IVec3, world: &Arc<World>) -> bool {
        let block_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let state = world.get_block_state(block_pos);
        let block = state.get_block();

        if block == &vanilla_blocks::AIR || block == &vanilla_blocks::VOID_AIR {
            return false;
        }

        let block_pos_below = BlockPos::new(pos.x, pos.y - 1, pos.z);
        let below_state = world.get_block_state(block_pos_below);
        let below_block = below_state.get_block();

        if below_block == &vanilla_blocks::AIR || below_block == &vanilla_blocks::VOID_AIR {
            if self.can_float.load(Ordering::Relaxed) {
                return block != &vanilla_blocks::AIR;
            }
            return false;
        }

        true
    }

    fn reconstruct_path(&self, came_from: &HashMap<IVec3, IVec3>, end: IVec3) -> Vec<PathNode> {
        let mut path = Vec::new();
        let mut current = end;

        while let Some(&prev) = came_from.get(&current) {
            path.push(PathNode::new_standalone(current.x, current.y, current.z));
            current = prev;
        }

        path.reverse();
        path
    }

    #[allow(unused)]
    pub fn get_path(&self) -> Option<Vec<PathNode>> {
        let path = self.path.lock();
        if path.is_empty() {
            None
        } else {
            Some(path.clone())
        }
    }

    pub fn is_done(&self) -> bool {
        let pos = self.entity.position();
        if let Some(target) = *self.target_pos.lock() {
            let dist = pos.distance(target);
            return dist < 1.0;
        }
        true
    }
}

#[derive(Clone)]
pub struct PathNode {
    x: i32,
    y: i32,
    z: i32,
    f: f64,
}

impl PathNode {
    pub fn new_standalone(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z, f: 0.0 }
    }

    pub fn with_f(x: i32, y: i32, z: i32, f: f64) -> Self {
        Self { x, y, z, f }
    }

    pub fn as_ivec(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.x == other.x && self.z == other.z
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn heuristic(a: IVec3, b: IVec3) -> f64 {
    let dx = (b.x - a.x) as f64;
    let dy = (b.y - a.y) as f64;
    let dz = (b.z - a.z) as f64;
    (dx * dx + dy * dy + dz * dz).sqrt()
}