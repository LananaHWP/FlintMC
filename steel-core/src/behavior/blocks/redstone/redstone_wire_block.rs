//! Redstone wire (dust) block behavior.
//!
//! Redstone wire propagates signals from power sources across the world.
//! Signal strength decays by 1 for each block of distance traveled.
//!
//! Vanilla equivalent: `RedStoneWireBlock`.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::REGISTRY;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, RedstoneSide};
use steel_registry::vanilla_blocks;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::BlockPlaceContext;
use crate::behavior::BLOCK_BEHAVIORS;
use crate::world::World;

/// Maximum redstone signal strength.
const MAX_SIGNAL: i32 = 15;

/// Minimum signal strength to consider the wire as "lit".
const MIN_SIGNAL: i32 = 1;

/// Behavior for redstone wire (redstone dust).
///
/// Redstone wire propagates power from sources, with signal decreasing by 1 per block.
#[block_behavior]
pub struct RedStoneWireBlock {
    block: BlockRef,
}

impl RedStoneWireBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Gets the power level from one of the 4 possible connections (N/S/E/W).
    fn get_power_from_direction(_state: BlockStateId, world: &Arc<World>, pos: BlockPos, direction: Direction) -> i32 {
        let neighbor_pos = direction.relative(pos);
        let neighbor_state = world.get_block_state(neighbor_pos);
        let neighbor_block = neighbor_state.get_block();
        let behavior = BLOCK_BEHAVIORS.get_behavior(neighbor_block);
        behavior.get_block_power(neighbor_state, world, neighbor_pos, direction.opposite())
    }

    /// Calculates the power this wire should emit based on its connections.
    fn calculate_power(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> i32 {
        let north: RedstoneSide = state.get_value(&BlockStateProperties::NORTH_REDSTONE);
        let south: RedstoneSide = state.get_value(&BlockStateProperties::SOUTH_REDSTONE);
        let east: RedstoneSide = state.get_value(&BlockStateProperties::EAST_REDSTONE);
        let west: RedstoneSide = state.get_value(&BlockStateProperties::WEST_REDSTONE);

        let mut max_power = 0;

        // Check each direction for a stronger signal
        if north != RedstoneSide::None {
            let power = Self::get_power_from_direction(state, world, pos, Direction::North);
            max_power = max_power.max(power - 1);
        }
        if south != RedstoneSide::None {
            let power = Self::get_power_from_direction(state, world, pos, Direction::South);
            max_power = max_power.max(power - 1);
        }
        if east != RedstoneSide::None {
            let power = Self::get_power_from_direction(state, world, pos, Direction::East);
            max_power = max_power.max(power - 1);
        }
        if west != RedstoneSide::None {
            let power = Self::get_power_from_direction(state, world, pos, Direction::West);
            max_power = max_power.max(power - 1);
        }

        max_power
    }

    /// Updates wire connections based on adjacent blocks.
    fn update_connections(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) -> BlockStateId {
        let mut new_state = state;

        // Check each direction
        let can_connect_north = self.can_connect_to(world, pos, Direction::North);
        let can_connect_south = self.can_connect_to(world, pos, Direction::South);
        let can_connect_east = self.can_connect_to(world, pos, Direction::East);
        let can_connect_west = self.can_connect_to(world, pos, Direction::West);

        let north_side = if can_connect_north { RedstoneSide::Up } else { RedstoneSide::None };
        let south_side = if can_connect_south { RedstoneSide::Up } else { RedstoneSide::None };
        let east_side = if can_connect_east { RedstoneSide::Up } else { RedstoneSide::None };
        let west_side = if can_connect_west { RedstoneSide::Up } else { RedstoneSide::None };

        new_state = new_state.set_value(&BlockStateProperties::NORTH_REDSTONE, north_side);
        new_state = new_state.set_value(&BlockStateProperties::SOUTH_REDSTONE, south_side);
        new_state = new_state.set_value(&BlockStateProperties::EAST_REDSTONE, east_side);
        new_state = new_state.set_value(&BlockStateProperties::WEST_REDSTONE, west_side);

        // Calculate and set the power
        let power = self.calculate_power(new_state, world, pos);
        new_state = new_state.set_value(&BlockStateProperties::POWER, power as u8);

        new_state
    }

    /// Checks if wire can connect to a block in the given direction.
    fn can_connect_to(&self, world: &Arc<World>, pos: BlockPos, direction: Direction) -> bool {
        let check_pos = direction.relative(pos);

        // Check if there's a wire directly above (wire always can connect to wire above)
        let above_pos = check_pos.above();
        let above_state = world.get_block_state(above_pos);
        if above_state.get_block() == &vanilla_blocks::REDSTONE_WIRE {
            return true;
        }

        // Check the block at the connection position
        let check_state = world.get_block_state(check_pos);
        let check_block = check_state.get_block();
        let behavior = BLOCK_BEHAVIORS.get_behavior(check_block);

        // Can connect to any block that provides a signal
        if behavior.is_signal_source(check_state) {
            return true;
        }

        // Check for opaque block that can provide power via get_block_power
        if behavior.get_block_power(check_state, world, check_pos, Direction::Up) > 0 {
            return true;
        }

        false
    }
}

impl BlockBehavior for RedStoneWireBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let state = self.block.default_state();
        Some(state.set_value(&BlockStateProperties::POWER, 0))
    }

    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        // Update connections and power after placement
        let new_state = self.update_connections(state, world, pos);
        if new_state != state {
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
        }
    }

    fn is_signal_source(&self, state: BlockStateId) -> bool {
        let power: u8 = state.get_value(&BlockStateProperties::POWER);
        power > 0
    }

    fn get_signal(&self, state: BlockStateId, _world: &Arc<World>, _pos: BlockPos) -> i32 {
        let power: u8 = state.get_value(&BlockStateProperties::POWER);
        power as i32
    }

    fn get_block_power(
        &self,
        state: BlockStateId,
        _world: &Arc<World>,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        let power: u8 = state.get_value(&BlockStateProperties::POWER);
        power as i32
    }

    fn handle_neighbor_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source_block: BlockRef,
        moved_by_piston: bool,
    ) {
        if moved_by_piston {
            return;
        }

        // Check if we should still exist (must connect to something)
        let can_connect = self.can_connect_to(world, pos, Direction::North)
            || self.can_connect_to(world, pos, Direction::South)
            || self.can_connect_to(world, pos, Direction::East)
            || self.can_connect_to(world, pos, Direction::West)
            || self.can_connect_to(world, pos, Direction::Up);

        if !can_connect {
            world.set_block(pos, REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR), UpdateFlags::UPDATE_NEIGHBORS);
            return;
        }

        // Update connections and recalculate power
        let new_state = self.update_connections(state, world, pos);
        let old_power: u8 = state.get_value(&BlockStateProperties::POWER);
        let new_power: u8 = new_state.get_value(&BlockStateProperties::POWER);

        if new_state != state {
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
        }

        // If power changed, notify neighbors
        if old_power != new_power {
            world.update_neighbors_at(pos, self.block);
        }
    }
}