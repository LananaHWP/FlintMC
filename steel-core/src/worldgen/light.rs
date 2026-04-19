//! Light propagation system for SteelMC.
//! 
//! This module implements light propagation similar to vanilla's LightEngine.
//! It handles both sky light (from the sun) and block light (from light-emitting blocks).
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::vanilla_blocks;
use steel_utils::BlockStateId;

pub const MAX_LIGHT: u8 = 15;
pub const EMPTY_LIGHT: u8 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Sky,
    Block,
}

pub fn get_light_emission(state: BlockStateId) -> u8 {
    let block = state.get_block();
    
    if block == &vanilla_blocks::TORCH
        || block == &vanilla_blocks::WALL_TORCH
        || block == &vanilla_blocks::SOUL_TORCH
        || block == &vanilla_blocks::SOUL_WALL_TORCH
    {
        return 12;
    }
    
    if block == &vanilla_blocks::GLOWSTONE
        || block == &vanilla_blocks::SHROOMLIGHT
    {
        return 15;
    }
    
    if block == &vanilla_blocks::BEACON {
        return 15;
    }
    
    if block == &vanilla_blocks::END_ROD
        || block == &vanilla_blocks::END_PORTAL
        || block == &vanilla_blocks::END_GATEWAY
    {
        return 15;
    }
    
    if block == &vanilla_blocks::LANTERN {
        return 15;
    }
    
    if block == &vanilla_blocks::SOUL_LANTERN {
        return 12;
    }
    
    if block == &vanilla_blocks::CANDLE {
        return 12;
    }
    
    if block == &vanilla_blocks::REDSTONE_LAMP
        || block == &vanilla_blocks::REDSTONE_BLOCK
    {
        return 15;
    }
    
    if block == &vanilla_blocks::JACK_O_LANTERN {
        return 15;
    }
    
    if block == &vanilla_blocks::CONDUIT {
        return 15;
    }
    
    if block == &vanilla_blocks::RESPAWN_ANCHOR {
        return 15;
    }
    
    if block == &vanilla_blocks::FIRE {
        return 15;
    }
    
    if block == &vanilla_blocks::SOUL_FIRE {
        return 15;
    }
    
    if block == &vanilla_blocks::MAGMA_BLOCK {
        return 15;
    }
    
    if block == &vanilla_blocks::BLAST_FURNACE
        || block == &vanilla_blocks::SMITHING_TABLE
        || block == &vanilla_blocks::FURNACE
        || block == &vanilla_blocks::SMOKER
    {
        return 13;
    }
    
    if block == &vanilla_blocks::BREWING_STAND {
        return 2;
    }
    
    0
}

pub fn is_transparent(state: BlockStateId) -> bool {
    let block = state.get_block();
    
    if state.is_air() {
        return true;
    }
    
    if block == &vanilla_blocks::AIR
        || block == &vanilla_blocks::CAVE_AIR
        || block == &vanilla_blocks::VOID_AIR
        || block == &vanilla_blocks::WATER
    {
        return true;
    }
    
    if block == &vanilla_blocks::GLASS {
        return true;
    }
    
    if block == &vanilla_blocks::SNOW
        || block == &vanilla_blocks::STRUCTURE_VOID
    {
        return true;
    }
    
    false
}

pub fn is_block_fully_transparent(state: BlockStateId) -> bool {
    state.is_air() || is_transparent(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_registry::{REGISTRY, vanilla_blocks};
    
    #[test]
    fn test_torch_emission() {
        let torch = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::TORCH);
        assert_eq!(get_light_emission(torch), 12);
    }
    
    #[test]
    fn test_glowstone_emission() {
        let glowstone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GLOWSTONE);
        assert_eq!(get_light_emission(glowstone), 15);
    }
    
    #[test]
    fn test_air_emission() {
        let air = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        assert_eq!(get_light_emission(air), 0);
    }
    
    #[test]
    fn test_air_transparent() {
        let air = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        assert!(is_transparent(air));
    }
}