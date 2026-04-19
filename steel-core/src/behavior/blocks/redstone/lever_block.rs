//! Lever block behavior.
//!
//! Levers are manually toggled redstone power sources that can be turned on/off.
//! They emit a redstone signal when powered (FACE = CEILING or FLOOR with
//! POWERED = true, or WALL_HORIZONTAL_FACING with POWERED = true).
//!
//! Vanilla equivalent: `Switch` + directional logic.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::REGISTRY;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::properties::{AttachFace, BlockStateProperties, Direction};
use steel_registry::vanilla_blocks;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::player::Player;
use crate::world::World;

/// Behavior for lever block variants.
///
/// Levers are simpler than buttons - they toggle on/off when clicked
/// and stay in that state until clicked again.
#[block_behavior]
pub struct LeverBlock {
    block: BlockRef,
}

impl LeverBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Returns the direction the lever is facing (away from mounting block).
    fn get_connected_direction(state: BlockStateId) -> Direction {
        let face: AttachFace = state.get_value(&BlockStateProperties::ATTACH_FACE);
        match face {
            AttachFace::Floor => Direction::Up,
            AttachFace::Ceiling => Direction::Down,
            AttachFace::Wall => state.get_value(&BlockStateProperties::HORIZONTAL_FACING),
        }
    }

    /// Updates neighbors at the lever position and the supporting block position.
    fn update_lever_neighbors(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        world.update_neighbors_at(pos, self.block);
        let support_dir = Self::get_connected_direction(state).opposite();
        let support_pos = support_dir.relative(pos);
        world.update_neighbors_at(support_pos, self.block);
    }

    /// Toggles the lever state.
    fn toggle(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos, player: &Player) {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        let new_powered = !powered;
        let new_state = state.set_value(&BlockStateProperties::POWERED, new_powered);
        world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
        self.update_lever_neighbors(new_state, world, pos);
    }
}

impl BlockBehavior for LeverBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        for direction in context.get_nearest_looking_directions() {
            let state = if direction.get_axis() == steel_utils::math::Axis::Y {
                let face = if direction == Direction::Up {
                    AttachFace::Ceiling
                } else {
                    AttachFace::Floor
                };
                self.block
                    .default_state()
                    .set_value(&BlockStateProperties::ATTACH_FACE, face)
                    .set_value(&BlockStateProperties::HORIZONTAL_FACING, context.horizontal_direction)
            } else {
                self.block
                    .default_state()
                    .set_value(&BlockStateProperties::ATTACH_FACE, AttachFace::Wall)
                    .set_value(&BlockStateProperties::HORIZONTAL_FACING, direction.opposite())
            };

            if self.can_survive(state, context.world, context.relative_pos) {
                return Some(state);
            }
        }
        None
    }

    fn can_survive(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> bool {
        let support_dir = Self::get_connected_direction(state).opposite();
        let support_pos = support_dir.relative(pos);
        let support_state = world.get_block_state(support_pos);
        support_state.is_face_sturdy(support_dir.opposite())
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let support_dir = Self::get_connected_direction(state).opposite();
        if direction == support_dir && !self.can_survive(state, world, pos) {
            return REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        }
        state
    }

    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
    ) -> InteractionResult {
        self.toggle(state, world, pos, player);
        InteractionResult::Success
    }

    fn is_signal_source(&self, state: BlockStateId) -> bool {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        powered
    }

    fn get_signal(&self, state: BlockStateId, _world: &Arc<World>, _pos: BlockPos) -> i32 {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        if powered { 15 } else { 0 }
    }

    fn affect_neighbors_after_removal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        moved_by_piston: bool,
    ) {
        if moved_by_piston {
            return;
        }
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        if !powered {
            return;
        }
        self.update_lever_neighbors(state, world, pos);
    }
}