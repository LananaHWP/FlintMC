//! The shulker box menu for portable storage.
//!
//! Slot layout (63 total):
//! - Slots 0-26: Shulker box container (27 slots)
//! - Slots 27-53: Main inventory (27 slots)
//! - Slots 54-62: Hotbar (9 slots)

use std::mem;

use steel_registry::item_stack::ItemStack;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_menu_types;
use steel_utils::BlockPos;
use text_components::TextComponent;

use crate::inventory::{
    SyncPlayerInv,
    lock::{ContainerLockGuard, ContainerRef},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, Slot, SlotType, add_standard_inventory_slots},
};
use crate::player::Player;

pub const SLOTS_PER_ROW: usize = 9;

pub mod slots {
    use super::SLOTS_PER_ROW;

    pub const fn container_slot_count(rows: usize) -> usize {
        rows * SLOTS_PER_ROW
    }

    pub const fn inv_slot_start(rows: usize) -> usize {
        container_slot_count(rows)
    }

    pub const fn inv_slot_end(rows: usize) -> usize {
        inv_slot_start(rows) + 27
    }

    pub const fn hotbar_slot_start(rows: usize) -> usize {
        inv_slot_end(rows)
    }

    pub const fn hotbar_slot_end(rows: usize) -> usize {
        hotbar_slot_start(rows) + 9
    }

    pub const fn total_slots(rows: usize) -> usize {
        hotbar_slot_end(rows)
    }
}

/// The shulker box menu for portable storage.
///
/// Based on Java's `ShulkerBoxMenu`.
pub struct ShulkerBoxMenu {
    behavior: MenuBehavior,
    container: ContainerRef,
    rows: usize,
}

impl ShulkerBoxMenu {
    #[must_use]
    pub fn new(
        inventory: SyncPlayerInv,
        container_id: u8,
        container: ContainerRef,
        rows: usize,
    ) -> Self {
        assert!((1..=3).contains(&rows), "Shulker box rows must be 1-3");

        let container_slots = slots::container_slot_count(rows);
        let total_slots = slots::total_slots(rows);
        let mut menu_slots = Vec::with_capacity(total_slots);

        // Container slots
        for i in 0..container_slots {
            menu_slots.push(SlotType::Normal(NormalSlot::new(container.clone(), i)));
        }

        // Main inventory (indices 9-35 -> menu 27-53)
        for i in 9..36 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i)));
        }

        // Hotbar (indices 0-8 -> menu 54-62)
        for i in 0..9 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i)));
        }

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::SHULKER_BOX),
            ),
            container,
            rows,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::SHULKER_BOX
    }

    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }
}

impl Menu for ShulkerBoxMenu {
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

        let container_slots = slots::container_slot_count(self.rows);
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < container_slots {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, container_slots, total_slots, true)
        } else {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, 0, container_slots, false)
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

    fn still_valid(&self) -> bool {
        let guard = self.behavior.lock_all_containers();
        guard.get(self.container.container_id()).is_some()
    }

    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }
    }
}

impl MenuInstance for ShulkerBoxMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::SHULKER_BOX
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating shulker box menus.
pub struct ShulkerBoxMenuProvider {
    inventory: SyncPlayerInv,
    container: ContainerRef,
    title: TextComponent,
}

impl ShulkerBoxMenuProvider {
    #[must_use]
    pub const fn new(
        inventory: SyncPlayerInv,
        container: ContainerRef,
        title: TextComponent,
    ) -> Self {
        Self {
            inventory,
            container,
            title,
        }
    }
}

impl MenuProvider for ShulkerBoxMenuProvider {
    fn title(&self) -> TextComponent {
        self.title.clone()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(ShulkerBoxMenu::new(
            self.inventory.clone(),
            container_id,
            self.container.clone(),
            3,
        ))
    }
}