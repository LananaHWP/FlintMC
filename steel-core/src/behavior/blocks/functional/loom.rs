//! Loom block behavior implementation.

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockPlaceContext, InteractionResult};
use crate::world::World;
use std::sync::Arc;

#[block_behavior]
pub struct LoomBlock {
    block: BlockRef,
}

impl LoomBlock {
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for LoomBlock {
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
}