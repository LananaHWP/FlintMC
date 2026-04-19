//! The brewing stand menu for potion brewing.
//!
//! Slot layout (38 total):
//! - Slot 0: Ingredient
//! - Slots 1-3: Bottles (3药水格)
//! - Slot 4: Fuel
//! - Slots 5-32: Main inventory (27 slots)
//! - Slots 33-41: Hotbar (9 slots)

use std::mem;

use steel_registry::item_stack::ItemStack;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_menu_types;
use steel_utils::BlockPos;
use steel_utils::translations;
use text_components::TextComponent;

use crate::inventory::{
    SyncPlayerInv,
    lock::{ContainerLockGuard, ContainerRef},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, Slot, SlotType, add_standard_inventory_slots},
};
use crate::player::Player;

/// Slot indices for the brewing stand menu.
pub mod slots {
    /// Slot index for the ingredient (slot 0).
    pub const INGREDIENT_SLOT: usize = 0;
    /// Start of the bottle slots (slot 1).
    pub const BOTTLE_SLOT_START: usize = 1;
    /// End of the bottle slots (slot 4, exclusive).
    pub const BOTTLE_SLOT_END: usize = 4;
    /// Slot index for the fuel (slot 4).
    pub const FUEL_SLOT: usize = 4;
    /// Start of main inventory (slot 5).
    pub const INV_SLOT_START: usize = 5;
    /// End of main inventory (slot 33, exclusive).
    pub const INV_SLOT_END: usize = 33;
    /// Start of hotbar (slot 33).
    pub const HOTBAR_SLOT_START: usize = 33;
    /// End of hotbar (slot 42, exclusive).
    pub const HOTBAR_SLOT_END: usize = 42;
    /// Total number of slots in the brewing stand menu.
    pub const TOTAL_SLOTS: usize = 42;
}

/// The brewing stand menu for potion brewing.
///
/// Based on Java's `BrewingStandMenu`.
pub struct BrewingStandMenu {
    behavior: MenuBehavior,
    block_pos: BlockPos,
}

impl BrewingStandMenu {
    /// Creates a new brewing stand menu.
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory
    /// * `container_id` - The container ID for this menu
    /// * `block_pos` - The position of the brewing stand block
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        // Slot 0: Ingredient
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slots 1-3: Bottles
        for i in 0..3 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 1)));
        }

        // Slot 4: Fuel
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 4)));

        // Slots 5-41: Standard inventory
        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::BREWING_STAND),
            ),
            block_pos,
        }
    }

    /// Returns the menu type for the brewing stand.
    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::BREWING_STAND
    }

    /// Returns the position of the brewing stand block.
    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl Menu for BrewingStandMenu {
    fn behavior(&self) -> &MenuBehavior {
        &self.behavior
    }

    fn behavior_mut(&mut self) -> &mut MenuBehavior {
        &mut self.behavior
    }

    /// Handles shift-click (quick move) for a slot.
    ///
    /// Based on Java's `BrewingStandMenu::quickMoveStack`:
    /// - Brewing slots (0-4) -> inventory (5-41)
    /// - Inventory (5-41) -> ingredient/fuel (0 or 4) or bottles (1-3)
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

        let brewing_slots = 5;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < brewing_slots {
            // Brewing slot -> player inventory
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                brewing_slots,
                total_slots,
                true,
            )
        } else {
            // Player inventory -> brewing slots
            // Try ingredient first (0), then bottles (1-3), then fuel (4)
            if !self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                0,
                1,
                false,
            ) {
                if !self.behavior.move_item_stack_to(
                    guard,
                    &mut stack_mut,
                    1,
                    4,
                    false,
                ) {
                    self.behavior.move_item_stack_to(
                        guard,
                        &mut stack_mut,
                        4,
                        5,
                        false,
                    )
                } else {
                    true
                }
            } else {
                true
            }
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

    /// Called when the brewing stand menu is closed.
    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }
    }
}

impl MenuInstance for BrewingStandMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::BREWING_STAND
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating brewing stand menus.
pub struct BrewingStandMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl BrewingStandMenuProvider {
    /// Creates a new brewing stand menu provider.
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for BrewingStandMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_BREWING.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(BrewingStandMenu::new(
            self.inventory.clone(),
            container_id,
            self.pos,
        ))
    }
}