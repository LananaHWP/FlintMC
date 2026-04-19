//! The hopper menu for item transfer.
//!
//! Slot layout (41 total):
//! - Slots 0-4: Hopper container slots (5)
//! - Slots 5-31: Main inventory (27 slots)
//! - Slots 32-40: Hotbar (9 slots)

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

/// Slot indices for the hopper menu.
pub mod slots {
    /// Number of hopper slots.
    pub const HOPPER_SLOTS: usize = 5;
    /// Start of main inventory (slot 5).
    pub const INV_SLOT_START: usize = 5;
    /// End of main inventory (slot 32, exclusive).
    pub const INV_SLOT_END: usize = 32;
    /// Start of hotbar (slot 32).
    pub const HOTBAR_SLOT_START: usize = 32;
    /// End of hotbar (slot 41, exclusive).
    pub const HOTBAR_SLOT_END: usize = 41;
    /// Total number of slots in the hopper menu.
    pub const TOTAL_SLOTS: usize = 41;
}

/// The hopper menu for item transfer.
///
/// Based on Java's `HopperMenu`.
pub struct HopperMenu {
    behavior: MenuBehavior,
    container: ContainerRef,
}

impl HopperMenu {
    /// Creates a new hopper menu.
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory
    /// * `container_id` - The container ID for this menu
    /// * `container` - Reference to the hopper container
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, container: ContainerRef) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        // First 5 slots are the hopper (container)
        for i in 0..slots::HOPPER_SLOTS {
            menu_slots.push(SlotType::Normal(NormalSlot::new(container.clone(), i)));
        }

        // Slots 5-40: Standard inventory
        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::HOPPER),
            ),
            container,
        }
    }

    /// Returns the menu type for the hopper.
    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::HOPPER
    }
}

impl Menu for HopperMenu {
    fn behavior(&self) -> &MenuBehavior {
        &self.behavior
    }

    fn behavior_mut(&mut self) -> &mut MenuBehavior {
        &mut self.behavior
    }

    /// Handles shift-click (quick move) for a slot.
    ///
    /// Based on Java's `HopperMenu::quickMoveStack`:
    /// - Hopper slots (0-4) -> inventory (5-40)
    /// - Inventory (5-40) -> hopper slots (0-4)
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

        let hopper_slots = slots::HOPPER_SLOTS;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < hopper_slots {
            // Hopper slot -> player inventory
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                hopper_slots,
                total_slots,
                true,
            )
        } else {
            // Player inventory -> hopper
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                0,
                hopper_slots,
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

    /// Called when the hopper menu is closed.
    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }
    }
}

impl MenuInstance for HopperMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::HOPPER
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating hopper menus.
pub struct HopperMenuProvider {
    inventory: SyncPlayerInv,
    container: ContainerRef,
    title: TextComponent,
}

impl HopperMenuProvider {
    /// Creates a new hopper menu provider.
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

impl MenuProvider for HopperMenuProvider {
    fn title(&self) -> TextComponent {
        self.title.clone()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(HopperMenu::new(
            self.inventory.clone(),
            container_id,
            self.container.clone(),
        ))
    }
}