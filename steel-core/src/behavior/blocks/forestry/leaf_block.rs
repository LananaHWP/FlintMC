//! Leaf block behavior implementation.
//!
//! Implements vanilla leaf decay: leaves decay (turn to dirt) when not near a log.
//! The decay logic checks within 6 blocks for logs (#minecraft:logs), and if none
//! are found after a random delay, leaves convert to dirt (or air with item drop).

use std::sync::Arc;
use steel_registry::blocks::Block;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::vanilla_blocks;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::world::World;

const DECAY_RADIUS: i32 = 6;
const CHECK_INTERVAL: i32 = 1;

fn is_leaf_block(block: &Block) -> bool {
    let path = &*block.key.path;
    path.ends_with("_leaves")
}

fn is_log(block: &Block) -> bool {
    let path = &*block.key.path;
    matches!(
        path,
        "oak_log" | "spruce_log" | "birch_log" | "jungle_log" | "acacia_log" | "dark_oak_log"
        | "cherry_log" | "mangrove_log" | "pale_oak_log" | "bamboo_log"
        | "crimson_stem" | "warped_stem"
        | "oak_wood" | "spruce_wood" | "birch_wood" | "jungle_wood" | "acacia_wood"
        | "dark_oak_wood" | "cherry_wood" | "mangrove_wood" | "pale_oak_wood"
        | "stripped_oak_log" | "stripped_spruce_log" | "stripped_birch_log" | "stripped_jungle_log"
        | "stripped_acacia_log" | "stripped_dark_oak_log" | "stripped_cherry_log"
        | "stripped_mangrove_log" | "stripped_pale_oak_log"
        | "stripped_oak_wood" | "stripped_spruce_wood" | "stripped_birch_wood"
        | "stripped_jungle_wood" | "stripped_acacia_wood" | "stripped_dark_oak_wood"
        | "stripped_cherry_wood" | "stripped_mangrove_wood" | "stripped_pale_oak_wood"
        | "crimson_hyphae" | "warped_hyphae"
        | "stripped_crimson_stem" | "stripped_warped_stem"
        | "stripped_crimson_hyphae" | "stripped_warped_hyphae"
        | "bamboo_block" | "bamboo_mosaic"
        | "bamboo_planks"
    ) || path.starts_with("stripped_bamboo")
}

fn has_log_nearby(world: &Arc<World>, pos: BlockPos) -> bool {
    let min_x = pos.x() - DECAY_RADIUS;
    let max_x = pos.x() + DECAY_RADIUS;
    let min_y = pos.y() - DECAY_RADIUS;
    let max_y = pos.y() + DECAY_RADIUS;
    let min_z = pos.z() - DECAY_RADIUS;
    let max_z = pos.z() + DECAY_RADIUS;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let check_pos = BlockPos::new(x, y, z);
                if check_pos != pos {
                    let neighbor_block = world.get_block_state(check_pos).get_block();
                    if is_log(neighbor_block) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub struct LeafBlock;

impl BlockBehavior for LeafBlock {
    fn get_state_for_placement(&self, _context: &crate::behavior::context::BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(vanilla_blocks::OAK_LEAVES.default_state())
    }

    fn is_randomly_ticking(&self, state: BlockStateId) -> bool {
        is_leaf_block(state.get_block())
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let block = state.get_block();
        
        if !is_leaf_block(block) {
            return;
        }

        let persistent = state
            .try_get_value(&BlockStateProperties::PERSISTENT)
            .unwrap_or(false);

        if persistent {
            return;
        }

        let distance = state
            .try_get_value(&BlockStateProperties::DISTANCE)
            .unwrap_or(7);

        if distance > 1 && !has_log_nearby(world, pos) {
            let new_distance = distance - 1;
            let new_state = state.set_value(&BlockStateProperties::DISTANCE, new_distance);
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);

            if new_distance > 1 {
                world.schedule_block_tick_default(
                    pos,
                    block,
                    CHECK_INTERVAL,
                );
            } else {
                world.set_block(
                    pos,
                    vanilla_blocks::DIRT.default_state(),
                    UpdateFlags::UPDATE_ALL,
                );
            }
        }
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.random_tick(state, world, pos);
    }
}