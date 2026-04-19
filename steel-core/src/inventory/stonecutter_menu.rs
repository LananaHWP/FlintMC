//! The stonecutter menu for stone variants.
//!
//! Slot layout (37 total):
//! - Slot 0: Input
//! - Slot 1: Output selection
//! - Slots 2-28: Output (result, virtual)
//! - Slots 29-37: Main inventory + hotbar

use std::mem;

use steel_registry::item_stack::ItemStack;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_menu_types;
use steel_utils::{BlockPos, translations};
use text_components::TextComponent;

use crate::inventory::{
    SyncPlayerInv,
    lock::{ContainerLockGuard, ContainerRef},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, Slot, SlotType, add_standard_inventory_slots},
};
use crate::player::Player;

pub mod slots {
    pub const INPUT: usize = 0;
    pub const INV_SLOT_START: usize = 1;
    pub const TOTAL_SLOTS: usize = 40;
}

pub struct StonecutterMenu {
    behavior: MenuBehavior,
    block_pos: BlockPos,
}

impl StonecutterMenu {
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        // Slot 0: Input
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slots 1-39: Standard inventory
        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::STONECUTTER),
            ),
            block_pos,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::STONECUTTER
    }
}

impl Menu for StonecutterMenu {
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

        let stonecutter_slots = 1;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < stonecutter_slots {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, stonecutter_slots, total_slots, true)
        } else {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, 0, stonecutter_slots, false)
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

    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }
    }
}

impl MenuInstance for StonecutterMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::STONECUTTER
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

pub struct StonecutterMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl StonecutterMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for StonecutterMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_STONECUTTER.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(StonecutterMenu::new(self.inventory.clone(), container_id, self.pos))
    }
}