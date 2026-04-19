//! The enchantment menu for selecting enchantments.
//!
//! Slot layout (37 total):
//! - Slot 0: Book slot (or item to enchant)
//! - Slots 1-27: Main inventory (27 slots)
//! - Slots 28-36: Hotbar (9 slots)

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

/// Slot indices for the enchantment menu.
pub mod slots {
    /// Slot index for the book/item (slot 0).
    pub const ENCHANT_SLOT: usize = 0;
    /// Start of main inventory (slot 1).
    pub const INV_SLOT_START: usize = 1;
    /// End of main inventory (slot 28, exclusive).
    pub const INV_SLOT_END: usize = 28;
    /// Start of hotbar (slot 28).
    pub const HOTBAR_SLOT_START: usize = 28;
    /// End of hotbar (slot 37, exclusive).
    pub const HOTBAR_SLOT_END: usize = 37;
    /// Total number of slots.
    pub const TOTAL_SLOTS: usize = 37;
}

/// The enchantment menu for selecting enchantments.
///
/// Based on Java's `EnchantmentMenu`.
pub struct EnchantmentMenu {
    behavior: MenuBehavior,
    block_pos: BlockPos,
    /// Enchantment power level.
    pub power: i32,
}

impl EnchantmentMenu {
    /// Creates a new enchantment menu.
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos, power: i32) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        // Slot 0: Enchantment target
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slots 1-36: Standard inventory
        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::ENCHANTMENT),
            ),
            block_pos,
            power,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::ENCHANTMENT
    }

    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }
}

impl Menu for EnchantmentMenu {
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

        let moved = if slot_index == slots::ENCHANT_SLOT {
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

impl MenuInstance for EnchantmentMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::ENCHANTMENT
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating enchantment menus.
pub struct EnchantmentMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
    power: i32,
}

impl EnchantmentMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos, power: i32) -> Self {
        Self { inventory, pos, power }
    }
}

impl MenuProvider for EnchantmentMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_ENCHANT.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(EnchantmentMenu::new(
            self.inventory.clone(),
            container_id,
            self.pos,
            self.power,
        ))
    }
}