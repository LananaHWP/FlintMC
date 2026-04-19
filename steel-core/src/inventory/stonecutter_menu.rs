//! The stonecutter menu for stone variants.
//!
//! Slot layout:
//! - Slot 0: Input (player inv slot 0)
//! - Slot 1: Output
//! - Slots 2-38: Player inventory + hotbar

use std::mem;
use std::sync::Arc;

use steel_registry::{
    item_stack::ItemStack,
    menu_type::MenuTypeRef,
    vanilla_menu_types,
    REGISTRY,
};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, translations};
use text_components::TextComponent;

use crate::inventory::{
    SyncPlayerInv,
    crafting::ResultContainer,
    lock::{ContainerId, ContainerLockGuard},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, ResultSlot, Slot, SlotType},
};
use crate::player::Player;

type SyncResultContainer = Arc<SyncMutex<ResultContainer>>;

pub mod slots {
    pub const INPUT: usize = 0;
    pub const OUTPUT: usize = 1;
    pub const INV_SLOT_START: usize = 2;
    pub const TOTAL_SLOTS: usize = 40;
}

pub struct StonecutterMenu {
    behavior: MenuBehavior,
    block_pos: BlockPos,
    result_container: SyncResultContainer,
}

impl StonecutterMenu {
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container = Arc::new(SyncMutex::new(ResultContainer::new()));

        // Slot 0: Input from player inventory (slot 0)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slot 1: Output (result slot)
        menu_slots.push(SlotType::Result(ResultSlot::new(
            result_container.clone(),
        )));

        // Slots 2-37: Player hotbar (slots 1-9)
        for i in 0..9 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 1)));
        }

        // Slots 38-39: Player offhand (slots 40-41)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 40)));
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 41)));

        // Additional player inventory slots (slots 9-35)
        for i in 9..36 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i)));
        }

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::STONECUTTER),
            ),
            block_pos,
            result_container,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::STONECUTTER
    }

    fn update_result(&self, guard: &mut ContainerLockGuard, inventory: &SyncPlayerInv) {
        let player_inv_id = ContainerId::from_arc(inventory);
        let input = guard.get(player_inv_id)
            .expect("player inventory not locked")
            .get_item(0);

        let result_stack = if input.is_empty() {
            ItemStack::empty()
        } else if let Some(recipe) = REGISTRY.recipes.find_stonecutter_recipe(input) {
            recipe.assemble()
        } else {
            ItemStack::empty()
        };

        guard.get_mut(ContainerId::from_arc(&self.result_container))
            .expect("result container not locked")
            .set_item(0, result_stack);
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

        // Handle output slot being taken
        if slot_index == slots::OUTPUT {
            if let Some(remainder) = slot.on_take(guard, &stack, player) {
                player.add_item_or_drop_with_guard(guard, remainder);
            }
            // Update result after taking - get inventory from slot 0
            let input_slot = &self.behavior.slots[slots::INPUT];
            let refs = input_slot.all_container_refs();
            for r in refs {
                if let crate::inventory::lock::ContainerRef::PlayerInventory(inv) = r {
                    self.update_result(guard, &inv);
                    break;
                }
            }
            return stack;
        }

        let clicked = stack.clone();
        let mut stack_mut = stack;

        let stonecutter_slots = 2;
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

        // Update result when input changes
        if slot_index == slots::INPUT {
            let input_slot = &self.behavior.slots[slots::INPUT];
            let refs = input_slot.all_container_refs();
            for r in refs {
                if let crate::inventory::lock::ContainerRef::PlayerInventory(inv) = r {
                    self.update_result(guard, &inv);
                    break;
                }
            }
        }

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