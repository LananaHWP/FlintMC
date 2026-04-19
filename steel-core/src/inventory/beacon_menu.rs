//! The beacon menu for setting beacon powers.
//!
//! Slot layout (37 total):
//! - Slot 0: Beacon payment slot
//! - Slots 1-27: Main inventory (27 slots)
//! - Slots 28-36: Hotbar (9 slots)

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

/// Slot indices for the beacon menu.
pub mod slots {
    /// Slot index for the beacon payment (slot 0).
    pub const PAYMENT_SLOT: usize = 0;
    /// Start of main inventory (slot 1).
    pub const INV_SLOT_START: usize = 1;
    /// End of main inventory (slot 28, exclusive).
    pub const INV_SLOT_END: usize = 28;
    /// Start of hotbar (slot 28).
    pub const HOTBAR_SLOT_START: usize = 28;
    /// End of hotbar (slot 37, exclusive).
    pub const HOTBAR_SLOT_END: usize = 37;
    /// Total number of slots in the beacon menu.
    pub const TOTAL_SLOTS: usize = 37;
}

/// The beacon menu for setting beacon powers.
///
/// Based on Java's `BeaconMenu`.
pub struct BeaconMenu {
    behavior: MenuBehavior,
    block_pos: BlockPos,
}

impl BeaconMenu {
    /// Creates a new beacon menu.
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory
    /// * `container_id` - The container ID for this menu
    /// * `block_pos` - The position of the beacon block
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        // Slot 0: Beacon payment
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slots 1-36: Standard inventory
        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::BEACON),
            ),
            block_pos,
        }
    }

    /// Returns the menu type for the beacon.
    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::BEACON
    }

    /// Returns the position of the beacon block.
    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl Menu for BeaconMenu {
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

        let moved = if slot_index == slots::PAYMENT_SLOT {
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                1,
                slots::TOTAL_SLOTS,
                true,
            )
        } else {
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                0,
                1,
                false,
            )
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

impl MenuInstance for BeaconMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::BEACON
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating beacon menus.
pub struct BeaconMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl BeaconMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for BeaconMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_BEACON.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(BeaconMenu::new(
            self.inventory.clone(),
            container_id,
            self.pos,
        ))
    }
}