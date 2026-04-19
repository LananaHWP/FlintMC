//! Comparator block behavior implementation.
//!
//! Comparators can compare two signals and output the difference or subtractions.
//! They also measure container contents.
//!
//! Vanilla equivalent: `ComparatorBlock`.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::ComparatorMode;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::properties::{BlockStateProperties, Direction};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::behavior::BLOCK_BEHAVIORS;
use crate::world::World;

#[block_behavior]
pub struct ComparatorBlock {
    block: BlockRef,
}

impl ComparatorBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    fn get_facing(&self, state: BlockStateId) -> Direction {
        state.get_value(&BlockStateProperties::HORIZONTAL_FACING)
    }

    fn get_input_pos(&self, state: BlockStateId, pos: BlockPos) -> BlockPos {
        let facing = self.get_facing(state);
        facing.opposite().relative(pos)
    }

    fn get_output_pos(&self, state: BlockStateId, pos: BlockPos) -> BlockPos {
        let facing = self.get_facing(state);
        facing.relative(pos)
    }

    fn get_input_power(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> i32 {
        let input_pos = self.get_input_pos(state, pos);
        let input_state = world.get_block_state(input_pos);
        let input_block = input_state.get_block();
        let behavior = BLOCK_BEHAVIORS.get_behavior(input_block);
        let facing = self.get_facing(state);
        behavior.get_block_power(input_state, world, input_pos, facing)
    }

    fn get_container_signal(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> i32 {
        let compare_pos = self.get_input_pos(state, pos);
        let compare_state = world.get_block_state(compare_pos);
        let compare_block = compare_state.get_block();
        let behavior = BLOCK_BEHAVIORS.get_behavior(compare_block);
        if behavior.has_analog_output_signal(compare_state) {
            return behavior.get_analog_output_signal(compare_state, world, compare_pos);
        }
        0
    }

    fn calculate_output(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> i32 {
        let input_power = self.get_input_power(state, world, pos);
        let container_signal = self.get_container_signal(state, world, pos);
        let mode: ComparatorMode = state.get_value(&BlockStateProperties::MODE_COMPARATOR);

        match mode {
            ComparatorMode::Subtract => (input_power - container_signal).max(0),
            ComparatorMode::Compare => {
                if input_power > container_signal || container_signal == 0 {
                    input_power
                } else {
                    0
                }
            }
        }
    }

    fn update_neighbors(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let output_pos = self.get_output_pos(state, pos);
        world.update_neighbors_at(output_pos, self.block);
    }
}

impl BlockBehavior for ComparatorBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let state = self
            .block
            .default_state()
            .set_value(&BlockStateProperties::HORIZONTAL_FACING, context.horizontal_direction);
        Some(state)
    }

    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _player: &crate::player::Player,
        _hit_result: &BlockHitResult,
    ) -> InteractionResult {
        let mode: ComparatorMode = state.get_value(&BlockStateProperties::MODE_COMPARATOR);
        let new_mode = match mode {
            ComparatorMode::Compare => ComparatorMode::Subtract,
            ComparatorMode::Subtract => ComparatorMode::Compare,
        };
        let new_state = state.set_value(&BlockStateProperties::MODE_COMPARATOR, new_mode);
        world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
        InteractionResult::Success
    }

    fn has_analog_output_signal(&self, state: BlockStateId) -> bool {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        powered
    }

    fn get_analog_output_signal(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> i32 {
        let output = self.calculate_output(state, world, pos);
        output
    }

    fn is_signal_source(&self, state: BlockStateId) -> bool {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        powered
    }

    fn get_signal(&self, state: BlockStateId, _world: &Arc<World>, _pos: BlockPos) -> i32 {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        if powered { 15 } else { 0 }
    }

    fn get_block_power(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        let output = self.calculate_output(state, world, pos);
        output
    }

    fn on_neighbor_signal_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source_block: BlockRef,
        moved_by_piston: bool,
    ) {
        if moved_by_piston {
            return;
        }

        let input_power = self.get_input_power(state, world, pos);
        let output = self.calculate_output(state, world, pos);
        let new_powered = output > 0;
        let current_powered: bool = state.get_value(&BlockStateProperties::POWERED);

        if current_powered != new_powered {
            let new_state = state.set_value(&BlockStateProperties::POWERED, new_powered);
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
            self.update_neighbors(new_state, world, pos);
        } else if output > 0 {
            self.update_neighbors(state, world, pos);
        }
    }
}