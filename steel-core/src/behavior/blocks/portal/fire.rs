//! Fire block behavior implementation.
//!
//! Vanilla splits fire into `BaseFireBlock` (portal logic, placement checks) and `FireBlock`
//! (spreading, aging). This combines the portal-relevant parts.

use std::sync::Arc;
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::game_rules::GameRuleValue;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_dimension_types;
use steel_registry::{REGISTRY, TaggedRegistryExt};
use steel_registry::vanilla_block_tags;
use steel_utils::math::Axis;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Direction};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::BlockPlaceContext;
use crate::entity::damage::DamageSource;
use crate::portal::portal_shape::{PortalShape, nether_portal_config};
use crate::world::tick_scheduler::TickPriority;
use crate::world::World;
use steel_registry::RegistryExt;
use steel_utils::Identifier;

const MAX_AGE: i32 = 15;
const MIN_AGE: i32 = 0;

#[inline]
fn keyeq(block: BlockRef, name: &str) -> bool {
    block.key.namespace == "minecraft" && block.key.path == name
}

static BURN_CHANCES: &[(&str, i32)] = &[
    ("minecraft:oak_planks", 5),
    ("minecraft:spruce_planks", 5),
    ("minecraft:birch_planks", 5),
    ("minecraft:jungle_planks", 5),
    ("minecraft:acacia_planks", 5),
    ("minecraft:dark_oak_planks", 5),
    ("minecraft:crimson_planks", 5),
    ("minecraft:warped_planks", 5),
    ("minecraft:oak_stairs", 5),
    ("minecraft:spruce_stairs", 5),
    ("minecraft:birch_stairs", 5),
    ("minecraft:jungle_stairs", 5),
    ("minecraft:acacia_stairs", 5),
    ("minecraft:dark_oak_stairs", 5),
    ("minecraft:crimson_stairs", 5),
    ("minecraft:warped_stairs", 5),
    ("minecraft:oak_slab", 5),
    ("minecraft:spruce_slab", 5),
    ("minecraft:birch_slab", 5),
    ("minecraft:jungle_slab", 5),
    ("minecraft:acacia_slab", 5),
    ("minecraft:dark_oak_slab", 5),
    ("minecraft:crimson_slab", 5),
    ("minecraft:warped_slab", 5),
    ("minecraft:oak_fence", 5),
    ("minecraft:spruce_fence", 5),
    ("minecraft:birch_fence", 5),
    ("minecraft:jungle_fence", 5),
    ("minecraft:acacia_fence", 5),
    ("minecraft:dark_oak_fence", 5),
    ("minecraft:crimson_fence", 5),
    ("minecraft:warped_fence", 5),
    ("minecraft:crimson_stem", 5),
    ("minecraft:warped_stem", 5),
    ("minecraft:stripped_oak_log", 5),
    ("minecraft:stripped_spruce_log", 5),
    ("minecraft:stripped_birch_log", 5),
    ("minecraft:stripped_jungle_log", 5),
    ("minecraft:stripped_acacia_log", 5),
    ("minecraft:stripped_dark_oak_log", 5),
    ("minecraft:stripped_crimson_stem", 5),
    ("minecraft:stripped_warped_stem", 5),
    ("minecraft:stripped_oak_wood", 5),
    ("minecraft:stripped_spruce_wood", 5),
    ("minecraft:stripped_birch_wood", 5),
    ("minecraft:stripped_jungle_wood", 5),
    ("minecraft:stripped_acacia_wood", 5),
    ("minecraft:stripped_dark_oak_wood", 5),
    ("minecraft:stripped_crimson_wood", 5),
    ("minecraft:stripped_warped_wood", 5),
    ("minecraft:oak_log", 5),
    ("minecraft:spruce_log", 5),
    ("minecraft:birch_log", 5),
    ("minecraft:jungle_log", 5),
    ("minecraft:acacia_log", 5),
    ("minecraft:dark_oak_log", 5),
    ("minecraft:oak_wood", 5),
    ("minecraft:spruce_wood", 5),
    ("minecraft:birch_wood", 5),
    ("minecraft:jungle_wood", 5),
    ("minecraft:acacia_wood", 5),
    ("minecraft:dark_oak_wood", 5),
    ("minecraft:nether_brick_slab", 5),
    ("minecraft:bookshelf", 5),
    ("minecraft:chest", 5),
    ("minecraft:daylight_detector", 5),
    ("minecraft:daylight_detector_inverted", 5),
    ("minecraft:lectern", 5),
    ("minecraft:loom", 5),
    ("minecraft:note_block", 5),
    ("minecraft:string", 5),
    ("minecraft:carpet", 5),
    ("minecraft:plant", 5),
    ("minecraft:lily_pad", 5),
    ("minecraft:scaffolding", 5),
    ("minecraft:lantern", 5),
    ("minecraft:candle", 5),
];

static SPREAD_CHANCES: &[(&str, i32)] = &[
    ("minecraft:oak_log", 5),
    ("minecraft:spruce_log", 5),
    ("minecraft:birch_log", 5),
    ("minecraft:jungle_log", 5),
    ("minecraft:acacia_log", 5),
    ("minecraft:dark_oak_log", 5),
    ("minecraft:stripped_oak_log", 5),
    ("minecraft:stripped_spruce_log", 5),
    ("minecraft:stripped_birch_log", 5),
    ("minecraft:stripped_jungle_log", 5),
    ("minecraft:stripped_acacia_log", 5),
    ("minecraft:stripped_dark_oak_log", 5),
    ("minecraft:stripped_crimson_stem", 5),
    ("minecraft:stripped_warped_stem", 5),
    ("minecraft:oak_wood", 5),
    ("minecraft:spruce_wood", 5),
    ("minecraft:birch_wood", 5),
    ("minecraft:jungle_wood", 5),
    ("minecraft:acacia_wood", 5),
    ("minecraft:dark_oak_wood", 5),
    ("minecraft:stripped_oak_wood", 5),
    ("minecraft:stripped_spruce_wood", 5),
    ("minecraft:stripped_birch_wood", 5),
    ("minecraft:stripped_jungle_wood", 5),
    ("minecraft:stripped_acacia_wood", 5),
    ("minecraft:stripped_dark_oak_wood", 5),
    ("minecraft:stripped_crimson_wood", 5),
    ("minecraft:stripped_warped_wood", 5),
    ("minecraft:crimson_stem", 5),
    ("minecraft:warped_stem", 5),
    ("minecraft:oak_planks", 5),
    ("minecraft:spruce_planks", 5),
    ("minecraft:birch_planks", 5),
    ("minecraft:jungle_planks", 5),
    ("minecraft:acacia_planks", 5),
    ("minecraft:dark_oak_planks", 5),
    ("minecraft:crimson_planks", 5),
    ("minecraft:warped_planks", 5),
    ("minecraft:bookshelf", 5),
    ("minecraft:cartography_table", 5),
    ("minecraft:crafter", 5),
    ("minecraft:daylight_detector", 5),
    ("minecraft:daylight_detector_inverted", 5),
    ("minecraft:lectern", 5),
    ("minecraft:loom", 5),
    ("minecraft:note_block", 5),
    ("minecraft:table", 5),
    ("minecraft:barrel", 5),
    ("minecraft:composter", 5),
];

/// Behavior for fire blocks.
#[block_behavior]
pub struct FireBlock {
    block: BlockRef,
}

impl FireBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    pub(crate) fn in_portal_dimension(world: &World) -> bool {
        world.dimension == &vanilla_dimension_types::OVERWORLD
            || world.dimension == &vanilla_dimension_types::THE_NETHER
    }

    pub(crate) fn can_be_placed_at(
        world: &Arc<World>,
        pos: BlockPos,
        forward_dir: Direction,
    ) -> bool {
        if !world.get_block_state(pos).is_air() {
            return false;
        }
        Self::can_survive_at(world, pos) || Self::is_portal(world, pos, forward_dir)
    }

    fn get_burn_chance(block: BlockRef) -> i32 {
        for (name, chance) in BURN_CHANCES {
            if keyeq(block, name) {
                return *chance;
            }
        }
        if REGISTRY.blocks.is_in_tag(block, &vanilla_block_tags::LOGS_TAG) {
            return 5;
        }
        if REGISTRY.blocks.is_in_tag(block, &vanilla_block_tags::LEAVES_TAG) {
            return 5;
        }
        0
    }

    fn get_spread_chance(block: BlockRef) -> i32 {
        for (name, chance) in SPREAD_CHANCES {
            if keyeq(block, name) {
                return *chance;
            }
        }
        if REGISTRY.blocks.is_in_tag(block, &vanilla_block_tags::LOGS_TAG) {
            return 5;
        }
        0
    }

    fn is_adjacent_flammable(world: &Arc<World>, pos: BlockPos) -> bool {
        for &dir in &Direction::ALL {
            let neighbor_pos = pos.relative(dir);
            let neighbor_block = world.get_block_state(neighbor_pos).get_block();
            if Self::get_burn_chance(neighbor_block) > 0 {
                return true;
            }
        }
        false
    }

    fn is_raining_around(world: &Arc<World>, pos: BlockPos) -> bool {
        let block_at = world.get_block_state(pos);
        if block_at.is_air() {
            return false;
        }
        for &dir in &[Direction::North, Direction::South, Direction::East, Direction::West] {
            let check_pos = pos.relative(dir).above();
            let block_at_pos = world.get_block_state(check_pos);
            if block_at_pos.get_block() == &vanilla_blocks::AIR {
                let sky_pos = pos.relative(dir).above().above();
                if world.get_block_state(sky_pos).is_air() {
                    return true;
                }
            }
        }
        false
    }

    fn get_spread_radius(world: &Arc<World>) -> i32 {
        let fire_spread_radius_key = Identifier::vanilla_static("fire_spread_radius");
        if let Some(rule) = REGISTRY.game_rules.by_key(&fire_spread_radius_key) {
            if let GameRuleValue::Int(val) = world.get_game_rule(rule) {
                return val.max(1).min(7);
            }
        }
        5
    }

    fn try_spread_fire(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        spread_factor: i32,
        current_age: i32,
    ) {
        let spread_radius = Self::get_spread_radius(world);
        if spread_radius <= 0 {
            return;
        }
        let should_rain = Self::is_raining_around(world, pos);

        for &dir in &Direction::ALL {
            let target_pos = pos.relative(dir);
            if world.get_block_state(target_pos).is_air() {
                if !world.get_block_state(target_pos.below()).is_air() {
                    let below_block = world.get_block_state(target_pos.below()).get_block();
                    let spread_chance = Self::get_spread_chance(below_block);
                    if spread_chance <= 0 {
                        continue;
                    }

                    if should_rain && rand::random_range(0i32..100) < spread_chance {
                        continue;
                    }

                    let distance_factor = spread_factor
                        * spread_chance
                        * (spread_radius * spread_radius);
                    if rand::random_range(0i32..10000) < distance_factor {
                        let below_key = format!("minecraft:{}", below_block.key.path);
                        let age = if Self::get_burn_chance(below_block) > 0 {
                            0
                        } else if below_key == "minecraft:soul_sand"
                            || below_key == "minecraft:soul_soil"
                        {
                            MIN_AGE
                        } else {
                            rand::random_range(0i32..MIN_AGE + 1)
                        };
                        world.set_block(
                            target_pos,
                            self.block.default_state(),
                            UpdateFlags::UPDATE_ALL,
                        );
                        world.schedule_block_tick(target_pos, &vanilla_blocks::FIRE, 1, TickPriority::Normal);
                    }
                }
            }
        }
    }

    fn get_state_with_age(world: &Arc<World>, pos: BlockPos, age: i32) -> BlockStateId {
        let new_age = age.clamp(MIN_AGE, MAX_AGE);
        let state = world.get_block_state(pos);
        state.set_value(&steel_registry::blocks::properties::BlockStateProperties::AGE_15, new_age as u8)
    }

    fn get_age(state: BlockStateId) -> i32 {
        state
            .try_get_value(&steel_registry::blocks::properties::BlockStateProperties::AGE_15)
            .unwrap_or(0) as i32
    }

    pub(crate) fn ignite_block(world: &Arc<World>, pos: BlockPos) {
        let current = world.get_block_state(pos).get_block();
        if current == &vanilla_blocks::AIR || current == &vanilla_blocks::FIRE {
            world.set_block(
                pos,
                vanilla_blocks::FIRE.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            world.schedule_block_tick(pos, &vanilla_blocks::FIRE, 1, TickPriority::Normal);
        }
    }

    fn can_survive_at(world: &Arc<World>, pos: BlockPos) -> bool {
        world
            .get_block_state(pos.below())
            .is_face_sturdy(Direction::Up)
            || Self::is_adjacent_flammable(world, pos)
    }

    fn is_portal(world: &Arc<World>, pos: BlockPos, forward_dir: Direction) -> bool {
        if !Self::in_portal_dimension(world) {
            return false;
        }

        let has_obsidian = Direction::ALL.iter().any(|&dir| {
            world.get_block_state(pos.relative(dir)).get_block() == &vanilla_blocks::OBSIDIAN
        });
        if !has_obsidian {
            return false;
        }

        let preferred_axis = if forward_dir.get_axis().is_horizontal() {
            forward_dir.rotate_y_counter_clockwise().get_axis()
        } else if rand::random::<bool>() {
            Axis::X
        } else {
            Axis::Z
        };

        let config = nether_portal_config();
        PortalShape::find_empty_portal_shape_with_axis(world, pos, preferred_axis, &config)
            .is_some()
    }
}

impl BlockBehavior for FireBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn can_survive(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> bool {
        Self::can_survive_at(world, pos)
    }

    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        if old_state.get_block() == state.get_block() {
            return;
        }

        if Self::in_portal_dimension(world)
            && let Some(shape) =
                PortalShape::find_empty_portal_shape(world, pos, &nether_portal_config())
        {
            shape.place_portal_blocks(world);
            return;
        }

        if !self.can_survive(state, world, pos) {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
        } else {
            world.schedule_block_tick(
                pos,
                &vanilla_blocks::FIRE,
                rand::random_range(20i32..40),
                TickPriority::Normal,
            );
        }
    }

    fn is_randomly_ticking(&self, _state: BlockStateId) -> bool {
        true
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let age = Self::get_age(state);

        if Self::is_raining_around(world, pos) {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            return;
        }

        let below = world.get_block_state(pos.below()).get_block();
        let below_key = format!("minecraft:{}", below.key.path);
        if below_key == "minecraft:soul_sand" || below_key == "minecraft:soul_soil" {
            return;
        }

        if below == &vanilla_blocks::AIR {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            return;
        }

        if Self::get_burn_chance(below) == 0 {
            if !Self::is_adjacent_flammable(world, pos) {
                let new_age = age + 1;
                if new_age > MAX_AGE {
                    world.set_block(
                        pos,
                        vanilla_blocks::AIR.default_state(),
                        UpdateFlags::UPDATE_ALL,
                    );
                    return;
                }
                let new_state = Self::get_state_with_age(world, pos, new_age);
                world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
            }
        }

        let spread_factor = if Self::get_spread_chance(below) > 0 { 1 } else { 2 };
        self.try_spread_fire(world, pos, spread_factor, age);
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.random_tick(state, world, pos);
    }

    fn entity_inside(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        entity: &dyn crate::entity::Entity,
    ) {
        let block = world.get_block_state(pos).get_block();
        if block == &vanilla_blocks::FIRE || block == &vanilla_blocks::SOUL_FIRE {
            entity.hurt(
                &DamageSource::environment(&steel_registry::vanilla_damage_types::IN_FIRE),
                1.0,
            );
        }
    }
}