//! Repeater block behavior implementation.
//!
//! Redstone repeaters delay signals by 1-4 ticks (configurable via DELAY property).
//! They also extend signal range and can be locked by another repeater input.
//!
//! Vanilla equivalent: `RepeaterBlock`.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::behavior::BLOCK_BEHAVIORS;
use crate::world::tick_scheduler::TickPriority;
use crate::world::World;

/// Tick delay per repeater delay level (in game ticks).
const DELAY_TICKS: i32 = 2;

/// Behavior for redstone repeater blocks.
///
/// Repeaters extend redstone signals and add a configurable delay.
#[block_behavior]
pub struct RepeaterBlock {
    block: BlockRef,
}

impl RepeaterBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Gets the delay in game ticks for this repeater state.
    fn get_delay(&self, state: BlockStateId) -> i32 {
        let delay: u8 = state.get_value(&BlockStateProperties::DELAY);
        ((delay as i32) + 1) * DELAY_TICKS
    }

    /// Gets the horizontal facing direction of the repeater.
    fn get_facing(&self, state: BlockStateId) -> Direction {
        state.get_value(&BlockStateProperties::HORIZONTAL_FACING)
    }

    /// Gets the position the input signal comes from.
    fn get_input_pos(&self, state: BlockStateId, pos: BlockPos) -> BlockPos {
        let facing = self.get_facing(state);
        let input_dir = facing.opposite();
        input_dir.relative(pos)
    }

    /// Gets the position the output signal goes to.
    fn get_output_pos(&self, state: BlockStateId, pos: BlockPos) -> BlockPos {
        let facing = self.get_facing(state);
        facing.relative(pos)
    }

    /// Gets the power received from the input side.
    fn get_input_power(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> i32 {
        let input_pos = self.get_input_pos(state, pos);
        let input_state = world.get_block_state(input_pos);
        let input_block = input_state.get_block();
        let behavior = BLOCK_BEHAVIORS.get_behavior(input_block);
        let facing = self.get_facing(state);
        behavior.get_block_power(input_state, world, input_pos, facing)
    }

    /// Updates neighbors to propagate the output signal.
    fn update_neighbors(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let output_pos = self.get_output_pos(state, pos);
        world.update_neighbors_at(output_pos, self.block);
    }
}

impl BlockBehavior for RepeaterBlock {
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
        player: &crate::player::Player,
        hit_result: &BlockHitResult,
    ) -> InteractionResult {
        // Click to change delay
        let current_delay: u8 = state.get_value(&BlockStateProperties::DELAY);
        let new_delay = (current_delay + 1) % 4;
        let new_state = state.set_value(&BlockStateProperties::DELAY, new_delay as u8);
        world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
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

    fn get_block_power(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        let powered: bool = state.get_value(&BlockStateProperties::POWERED);
        if powered {
            15
        } else {
            0
        }
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

        let current_power: bool = state.get_value(&BlockStateProperties::POWERED);
        let input_power = self.get_input_power(state, world, pos);
        let new_powered = input_power > 0;

        // Only update if state changed and not already scheduled
        if current_power != new_powered {
            let delay = self.get_delay(state);
            world.schedule_block_tick(pos, self.block, delay, TickPriority::Normal);
        }
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let input_power = self.get_input_power(state, world, pos);
        let new_powered = input_power > 0;
        let current_powered: bool = state.get_value(&BlockStateProperties::POWERED);

        if current_powered != new_powered {
            let new_state = state.set_value(&BlockStateProperties::POWERED, new_powered);
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
            self.update_neighbors(new_state, world, pos);
        }
    }

    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        // Check input on placement
        let input_power = self.get_input_power(state, world, pos);
        let powered = input_power > 0;
        if powered {
            let delay = self.get_delay(state);
            world.schedule_block_tick(pos, self.block, delay, TickPriority::Normal);
        }
    }
}