//! The furnace menu for smelting/cooking.
//!
//! Slot layout (36 total):
//! - Slot 0: Input
//! - Slot 1: Fuel
//! - Slot 2: Output (result)
//! - Slots 3-29: Main inventory (27 slots)
//! - Slots 30-38: Hotbar (9 slots)

use std::mem;
use std::sync::Arc;

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

/// Slot indices for the furnace menu.
pub mod slots {
    /// Slot index for the input (slot 0).
    pub const INPUT_SLOT: usize = 0;
    /// Slot index for the fuel (slot 1).
    pub const FUEL_SLOT: usize = 1;
    /// Slot index for the output (slot 2).
    pub const OUTPUT_SLOT: usize = 2;
    /// Start of main inventory (slot 3).
    pub const INV_SLOT_START: usize = 3;
    /// End of main inventory (slot 30, exclusive).
    pub const INV_SLOT_END: usize = 30;
    /// Start of hotbar (slot 30).
    pub const HOTBAR_SLOT_START: usize = 30;
    /// End of hotbar (slot 39, exclusive).
    pub const HOTBAR_SLOT_END: usize = 39;
    /// Total number of slots in the furnace menu.
    pub const TOTAL_SLOTS: usize = 39;
}

/// A synchronized result container for furnace output.
pub type SyncFurnaceResultContainer = Arc<SyncMutex<ResultContainer>>;

/// The furnace menu for smelting/cooking.
///
/// Based on Java's `FurnaceMenu`.
pub struct FurnaceMenu {
    behavior: MenuBehavior,
    /// The result container for output.
    result_container: SyncFurnaceResultContainer,
    /// The position of the furnace block.
    block_pos: BlockPos,
    /// Menu type (furnace, blast_furnace, or smoker).
    menu_type_ref: MenuTypeRef,
}

impl FurnaceMenu {
    /// Creates a new furnace menu.
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory
    /// * `container_id` - The container ID for this menu
    /// * `block_pos` - The position of the furnace block
    /// * `menu_type` - The menu type (furnace, blast_furnace, or smoker)
    #[must_use]
    pub fn new(
        inventory: SyncPlayerInv,
        container_id: u8,
        block_pos: BlockPos,
        menu_type: MenuTypeRef,
    ) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container: SyncFurnaceResultContainer =
            Arc::new(SyncMutex::new(ResultContainer::new()));

        // Slot 0: Input
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slot 1: Fuel
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 1)));

        // Slot 2: Output (result slot - no placement allowed)
        menu_slots.push(SlotType::Result(ResultSlot::new(result_container.clone())));

        // Slots 3-38: Standard inventory (main inventory + hotbar)
        // Note: offset by 3 since first 3 slots are furnace slots
        for i in 0..27 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 3)));
        }
        for i in 0..9 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 30)));
        }

        Self {
            behavior: MenuBehavior::new(menu_slots, container_id, Some(menu_type)),
            result_container,
            block_pos,
            menu_type_ref: menu_type,
        }
    }

    /// Creates a new blast furnace menu.
    #[must_use]
    pub fn new_blast_furnace(
        inventory: SyncPlayerInv,
        container_id: u8,
        block_pos: BlockPos,
    ) -> Self {
        Self::new(
            inventory,
            container_id,
            block_pos,
            &vanilla_menu_types::BLAST_FURNACE,
        )
    }

    /// Creates a new smoker menu.
    #[must_use]
    pub fn new_smoker(
        inventory: SyncPlayerInv,
        container_id: u8,
        block_pos: BlockPos,
    ) -> Self {
        Self::new(
            inventory,
            container_id,
            block_pos,
            &vanilla_menu_types::SMOKER,
        )
    }

    /// Returns the menu type for the furnace menu.
    #[must_use]
    pub const fn menu_type(&self) -> MenuTypeRef {
        self.menu_type_ref
    }

    /// Returns the position of the furnace block.
    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }

    /// Returns a reference to the result container.
    #[must_use]
    pub const fn result_container(&self) -> &SyncFurnaceResultContainer {
        &self.result_container
    }
}

impl Menu for FurnaceMenu {
    fn behavior(&self) -> &MenuBehavior {
        &self.behavior
    }

    fn behavior_mut(&mut self) -> &mut MenuBehavior {
        &mut self.behavior
    }

    /// Handles shift-click (quick move) for a slot.
    ///
    /// Based on Java's `FurnaceMenu::quickMoveStack`:
    /// - Furnace slots (0-2) -> inventory (3-39)
    /// - Inventory (3-39) -> furnace input/fuel (0-1)
    fn quick_move_stack(
        &mut self,
        guard: &mut ContainerLockGuard,
        slot_index: usize,
        player: &Player,
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

        let container_slots = 3; // input, fuel, output
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < container_slots {
            // Furnace slot -> player inventory
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                container_slots,
                total_slots,
                true,
            )
        } else {
            // Player inventory -> furnace (input or fuel only)
            // Try both input and fuel slots
            if !self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                0,
                2,
                false,
            ) {
                // If both full, try other inventory
                false
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

        // Handle output slot being taken
        if slot_index == slots::OUTPUT_SLOT {
            if let Some(remainder) = slot.on_take(guard, &clicked, player) {
                player.add_item_or_drop_with_guard(guard, remainder);
            }
        }

        let _ = player;

        clicked
    }

    /// Returns true if the item can be taken from the slot during pickup all.
    fn can_take_item_for_pick_all(&self, _carried: &ItemStack, slot_index: usize) -> bool {
        slot_index != slots::OUTPUT_SLOT
    }

    /// Called when the furnace menu is closed.
    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }

        // Clear the output slot
        self.result_container.lock().set_item(0, ItemStack::empty());
    }
}

impl MenuInstance for FurnaceMenu {
    fn menu_type(&self) -> MenuTypeRef {
        self.menu_type_ref
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating furnace menus.
pub struct FurnaceMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
    menu_type: MenuTypeRef,
    title: TextComponent,
}

impl FurnaceMenuProvider {
    /// Creates a new furnace menu provider.
    #[must_use]
    pub const fn new(
        inventory: SyncPlayerInv,
        pos: BlockPos,
        menu_type: MenuTypeRef,
        title: TextComponent,
    ) -> Self {
        Self {
            inventory,
            pos,
            menu_type,
            title,
        }
    }

    /// Creates a provider for a blast furnace.
    #[must_use]
    pub fn blast_furnace(
        inventory: SyncPlayerInv,
        pos: BlockPos,
    ) -> Self {
        Self::new(
            inventory,
            pos,
            &vanilla_menu_types::BLAST_FURNACE,
            translations::CONTAINER_BLAST_FURNACE.msg().into(),
        )
    }

    /// Creates a provider for a smoker.
    #[must_use]
    pub fn smoker(
        inventory: SyncPlayerInv,
        pos: BlockPos,
    ) -> Self {
        Self::new(
            inventory,
            pos,
            &vanilla_menu_types::SMOKER,
            translations::CONTAINER_SMOKER.msg().into(),
        )
    }
}

impl MenuProvider for FurnaceMenuProvider {
    fn title(&self) -> TextComponent {
        self.title.clone()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(FurnaceMenu::new(
            self.inventory.clone(),
            container_id,
            self.pos,
            self.menu_type,
        ))
    }
}

/// The standard furnace menu (for regular furnaces).
pub type FurnaceMenuStandard = FurnaceMenu;

/// Provider for creating standard furnace menus.
pub type FurnaceMenuProviderStandard = FurnaceMenuProvider;