//! The smoker menu for smoking food.
//!
//! Slot layout (39 total):
//! - Slot 0: Input
//! - Slot 1: Fuel
//! - Slot 2: Output
//! - Slots 3-38: Player inventory

use std::mem;

use steel_registry::item_stack::ItemStack;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_menu_types;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, translations};
use text_components::TextComponent;

use crate::inventory::{
    SyncPlayerInv,
    container::Container,
    crafting::ResultContainer,
    lock::{ContainerLockGuard, ContainerRef},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, ResultSlot, Slot, SlotType, SyncResultContainer, add_standard_inventory_slots},
};
use crate::player::Player;
use std::sync::Arc;

pub type SyncSmokerResultContainer = Arc<SyncMutex<ResultContainer>>;

/// The smoker menu for smoking food.
/// This is essentially the same as a furnace, but uses the smoker menu type.
pub type SmokerMenuStandard = crate::inventory::FurnaceMenuStandard;

/// Provider for creating smoker menus.
pub type SmokerMenuProviderStandard = crate::inventory::FurnaceMenuProviderStandard;