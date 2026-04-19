//! The anvil menu for repairing and renaming items.
//!
//! Slot layout (39 total):
//! - Slot 0: Left input (item to repair/rename)
//! - Slot 1: Right input (sacrifice item or name tag)
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
    slot::{NormalSlot, ResultSlot, Slot, SlotType, SyncResultContainer},
};
use crate::player::Player;
use crate::inventory::crafting::ResultContainer;
use std::sync::Arc;
use steel_utils::locks::SyncMutex;

/// Slot indices for the anvil menu.
pub mod slots {
    /// Slot index for the left input (slot 0).
    pub const LEFT_INPUT: usize = 0;
    /// Slot index for the right input (slot 1).
    pub const RIGHT_INPUT: usize = 1;
    /// Slot index for the output (slot 2).
    pub const OUTPUT: usize = 2;
    /// Start of main inventory (slot 3).
    pub const INV_SLOT_START: usize = 3;
    /// End of main inventory (slot 31, exclusive).
    pub const INV_SLOT_END: usize = 31;
    /// Start of hotbar (slot 31).
    pub const HOTBAR_SLOT_START: usize = 31;
    /// End of hotbar (slot 40, exclusive).
    pub const HOTBAR_SLOT_END: usize = 40;
    /// Total number of slots in the anvil menu.
    pub const TOTAL_SLOTS: usize = 40;
}

/// A synchronized result container for anvil output.
pub type SyncAnvilResultContainer = Arc<SyncMutex<ResultContainer>>;

/// The anvil menu for repairing and renaming items.
///
/// Based on Java's `AnvilMenu`.
pub struct AnvilMenu {
    behavior: MenuBehavior,
    result_container: SyncAnvilResultContainer,
    block_pos: BlockPos,
    /// The repair cost to display to the player.
    pub cost: i32,
}

impl AnvilMenu {
    /// Creates a new anvil menu.
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory
    /// * `container_id` - The container ID for this menu
    /// * `block_pos` - The position of the anvil block
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container: SyncAnvilResultContainer =
            Arc::new(SyncMutex::new(ResultContainer::new()));

        // Slot 0: Left input (item to repair/rename)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slot 1: Right input (sacrifice item or name tag)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 1)));

        // Slot 2: Output (result slot - no placement allowed)
        menu_slots.push(SlotType::Result(ResultSlot::new(result_container.clone())));

        // Slots 3-39: Standard inventory (main + hotbar)
        for i in 0..27 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 3)));
        }
        for i in 0..9 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 30)));
        }

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::ANVIL),
            ),
            result_container,
            block_pos,
            cost: 0,
        }
    }

    /// Returns the menu type for the anvil.
    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::ANVIL
    }

    /// Returns the position of the anvil block.
    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl Menu for AnvilMenu {
    fn behavior(&self) -> &MenuBehavior {
        &self.behavior
    }

    fn behavior_mut(&mut self) -> &mut MenuBehavior {
        &mut self.behavior
    }

    /// Handles shift-click (quick move) for a slot.
    ///
    /// Based on Java's `AnvilMenu::quickMoveStack`:
    /// - Anvil slots (0-2) -> inventory (3-39)
    /// - Inventory (3-39) -> anvil slots (0-1)
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

        let anvil_slots = 3;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < anvil_slots {
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                anvil_slots,
                total_slots,
                true,
            )
        } else {
            self.behavior.move_item_stack_to(
                guard,
                &mut stack_mut,
                0,
                anvil_slots,
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

    /// Returns true if the item can be taken from the slot during pickup all.
    fn can_take_item_for_pick_all(&self, _carried: &ItemStack, slot_index: usize) -> bool {
        slot_index != slots::OUTPUT
    }

    /// Called when the anvil menu is closed.
    fn removed(&mut self, player: &Player) {
        let carried = mem::take(&mut self.behavior.carried);
        if !carried.is_empty() {
            player.drop_item(carried, false, true);
        }
        self.result_container.lock().set_item(0, ItemStack::empty());
    }
}

impl MenuInstance for AnvilMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::ANVIL
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating anvil menus.
pub struct AnvilMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl AnvilMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for AnvilMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_REPAIR.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(AnvilMenu::new(
            self.inventory.clone(),
            container_id,
            self.pos,
        ))
    }
}