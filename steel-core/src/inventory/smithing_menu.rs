//! The smithing menu for upgrading items.
//!
//! Slot layout (39 total):
//! - Slot 0: Input (item to upgrade)
//! - Slot 1: Addition (material)
//! - Slot 2: Output (result)
//! - Slots 3-30: Main inventory (27 slots)
//! - Slots 31-38: Hotbar (9 slots)

use std::mem;

use steel_registry::item_stack::ItemStack;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_menu_types;
use steel_utils::{BlockPos, translations};
use text_components::TextComponent;

use crate::inventory::{
    SyncPlayerInv,
    container::Container,
    lock::{ContainerLockGuard, ContainerRef},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, ResultSlot, Slot, SlotType, SyncResultContainer, add_standard_inventory_slots},
};
use crate::player::Player;
use std::sync::Arc;
use steel_utils::locks::SyncMutex;
use crate::inventory::crafting::ResultContainer;

/// Slot indices for the smithing menu.
pub mod slots {
    pub const INPUT: usize = 0;
    pub const ADDITION: usize = 1;
    pub const OUTPUT: usize = 2;
    pub const INV_SLOT_START: usize = 3;
    pub const INV_SLOT_END: usize = 31;
    pub const HOTBAR_SLOT_START: usize = 31;
    pub const HOTBAR_SLOT_END: usize = 40;
    pub const TOTAL_SLOTS: usize = 40;
}

pub type SyncSmithingResultContainer = Arc<SyncMutex<ResultContainer>>;

/// The smithing menu for upgrading items.
///
/// Based on Java's `SmithingMenu`.
pub struct SmithingMenu {
    behavior: MenuBehavior,
    result_container: SyncSmithingResultContainer,
    block_pos: BlockPos,
}

impl SmithingMenu {
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container: SyncSmithingResultContainer =
            Arc::new(SyncMutex::new(ResultContainer::new()));

        // Slots 0-1: Input and addition
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 1)));

        // Slot 2: Output
        menu_slots.push(SlotType::Result(ResultSlot::new(result_container.clone())));

        // Slots 3-39: Standard inventory
        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::SMITHING),
            ),
            result_container,
            block_pos,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::SMITHING
    }

    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl Menu for SmithingMenu {
    fn behavior(&self) -> &MenuBehavior {
        &self.behavior
    }

    fn behavior_mut(&mut self) -> &mut MenuBehavior {
        &mut self.behavior
    }

    fn quick_move_stack(
        &mut self,
        guard: &mut ContainerLockGuard,
        slot_index: usize,
        _player: &Player,
    ) -> ItemStack {
        if slot_index >= self.behavior.slots.len() {
            return ItemStack::empty();
        }

        let slot = &self.behavior.slots[slot_index];
        let stack = slot.get_item(guard).clone();
        if stack.is_empty() {
            return ItemStack::empty();
        }

        let clicked = stack.clone();
        let mut stack_mut = stack;

        let smithing_slots = 3;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < smithing_slots {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, smithing_slots, total_slots, true)
        } else {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, 0, smithing_slots, false)
        };

        if !moved {
            return ItemStack::empty();
        }

        self.behavior.slots[slot_index].set_item(guard, stack_mut.clone());
        if stack_mut.count == clicked.count {
            return ItemStack::empty();
        }
        self.behavior.slots[slot_index].set_changed(guard);
        clicked
    }

    fn can_take_item_for_pick_all(&self, _carried: &ItemStack, slot_index: usize) -> bool {
        slot_index != slots::OUTPUT
    }

    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }
        self.result_container.lock().set_item(0, ItemStack::empty());
    }
}

impl MenuInstance for SmithingMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::SMITHING
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating smithing menus.
pub struct SmithingMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl SmithingMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for SmithingMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_UPGRADE.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(SmithingMenu::new(self.inventory.clone(), container_id, self.pos))
    }
}