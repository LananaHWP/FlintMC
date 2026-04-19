//! The merchant menu for villager trading.
//!
//! Slot layout (39 total):
//! - Slot 0: Trade slot A (input)
//! - Slot 1: Trade slot B (input, optional)
//! - Slot 2: Output (result)
//! - Slots 3-29: Main inventory
//! - Slots 30-38: Hotbar

use std::mem;

use steel_registry::item_stack::ItemStack;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_menu_types;
use steel_utils::BlockPos;
use steel_utils::translations;
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

pub mod slots {
    pub const TRADE_A: usize = 0;
    pub const TRADE_B: usize = 1;
    pub const OUTPUT: usize = 2;
    pub const TOTAL_SLOTS: usize = 39;
}

pub type SyncMerchantResultContainer = Arc<SyncMutex<ResultContainer>>;

pub struct MerchantMenu {
    behavior: MenuBehavior,
    result_container: SyncMerchantResultContainer,
    entity_id: i64,
    #[allow(dead_code)]
    career: i32,
}

impl MerchantMenu {
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, entity_id: i64, career: i32) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container: SyncMerchantResultContainer =
            Arc::new(SyncMutex::new(ResultContainer::new()));

        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 1)));
        menu_slots.push(SlotType::Result(ResultSlot::new(result_container.clone())));

        add_standard_inventory_slots(&mut menu_slots, &inventory);

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::MERCHANT),
            ),
            result_container,
            entity_id,
            career,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::MERCHANT
    }
}

impl Menu for MerchantMenu {
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

        let merchant_slots = 3;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < merchant_slots {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, merchant_slots, total_slots, true)
        } else {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, 0, merchant_slots, false)
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

impl MenuInstance for MerchantMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::MERCHANT
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

pub struct MerchantMenuProvider {
    inventory: SyncPlayerInv,
    entity_id: i64,
    career: i32,
}

impl MerchantMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, entity_id: i64, career: i32) -> Self {
        Self { inventory, entity_id, career }
    }
}

impl MenuProvider for MerchantMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_BREWING.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(MerchantMenu::new(
            self.inventory.clone(),
            container_id,
            self.entity_id,
            self.career,
        ))
    }
}