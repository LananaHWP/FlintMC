//! Jukebox block behavior implementation.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockPlaceContext, InteractionResult};
use crate::block_entity::{BLOCK_ENTITIES, SharedBlockEntity};
use crate::world::World;

#[block_behavior]
pub struct JukeboxBlock {
    block: BlockRef,
}

impl JukeboxBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for JukeboxBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        _world: &Arc<World>,
        _pos: BlockPos,
        _player: &crate::player::Player,
        _hit_result: &crate::behavior::context::BlockHitResult,
    ) -> InteractionResult {
        InteractionResult::Pass
    }

    fn has_block_entity(&self) -> bool {
        true
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Option<SharedBlockEntity> {
        BLOCK_ENTITIES.create(
            &steel_registry::vanilla_block_entity_types::JUKEBOX,
            level,
            pos,
            state,
        )
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }
}