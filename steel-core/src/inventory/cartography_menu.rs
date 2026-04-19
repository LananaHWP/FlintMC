//! The cartography menu for map extension.
//!
//! Slot layout (37 total):
//! - Slot 0: Map
//! - Slot 1: Paper
//! - Slot 2: Output
//! - Slots 3-30: Main inventory
//! - Slots 31-38: Hotbar

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
    slot::{NormalSlot, ResultSlot, Slot, SlotType, add_standard_inventory_slots},
};
use crate::player::Player;
use std::sync::Arc;
use steel_utils::locks::SyncMutex;
use crate::inventory::crafting::ResultContainer;

pub mod slots {
    pub const MAP: usize = 0;
    pub const PAPER: usize = 1;
    pub const OUTPUT: usize = 2;
    pub const TOTAL_SLOTS: usize = 40;
}

pub type SyncCartographyResultContainer = Arc<SyncMutex<ResultContainer>>;

pub struct CartographyMenu {
    behavior: MenuBehavior,
    result_container: SyncCartographyResultContainer,
    block_pos: BlockPos,
}

impl CartographyMenu {
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container: SyncCartographyResultContainer =
            Arc::new(SyncMutex::new(ResultContainer::new()));

        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 1)));
        menu_slots.push(SlotType::Result(ResultSlot::new(result_container.clone())));

        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::CARTOGRAPHY_TABLE),
            ),
            result_container,
            block_pos,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::CARTOGRAPHY_TABLE
    }
}

impl Menu for CartographyMenu {
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

        let carto_slots = 3;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < carto_slots {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, carto_slots, total_slots, true)
        } else {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, 0, carto_slots, false)
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

impl MenuInstance for CartographyMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::CARTOGRAPHY_TABLE
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

pub struct CartographyMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl CartographyMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for CartographyMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_CARTOGRAPHY_TABLE.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(CartographyMenu::new(self.inventory.clone(), container_id, self.pos))
    }
}