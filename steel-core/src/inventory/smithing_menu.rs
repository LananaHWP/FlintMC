//! The smithing menu for upgrading items.
//!
//! Slot layout (40 total):
//! - Slot 0: Template (armor trim template or upgrade template)
//! - Slot 1: Base (armor piece or tool/weapon)
//! - Slot 2: Addition (trim material or netherite ingot)
//! - Slot 3: Output (result)
//! - Slots 4-30: Main inventory (27 slots)
//! - Slots 31-38: Hotbar (9 slots)
//! - Slot 39: Offhand

use std::mem;
use std::sync::Arc;

use steel_registry::{
    item_stack::ItemStack,
    menu_type::MenuTypeRef,
    recipe::SmithingRecipeResult,
    REGISTRY,
    vanilla_menu_types,
};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, translations};
use text_components::TextComponent;

use crate::inventory::{
    container::Container,
    crafting::ResultContainer,
    lock::{ContainerId, ContainerLockGuard, ContainerRef, SyncPlayerInv},
    menu::{Menu, MenuBehavior},
    menu_provider::{MenuInstance, MenuProvider},
    slot::{NormalSlot, ResultSlot, Slot, SlotType},
};
use crate::player::Player;

/// Slot indices for the smithing menu.
pub mod slots {
    pub const TEMPLATE: usize = 0;
    pub const BASE: usize = 1;
    pub const ADDITION: usize = 2;
    pub const OUTPUT: usize = 3;
    pub const INV_SLOT_START: usize = 4;
    pub const INV_SLOT_END: usize = 31;
    pub const HOTBAR_SLOT_START: usize = 31;
    pub const HOTBAR_SLOT_END: usize = 39;
    pub const TOTAL_SLOTS: usize = 40;
}

pub type SyncSmithingResultContainer = Arc<SyncMutex<ResultContainer>>;

/// The smithing menu for upgrading items.
///
/// Based on Java's `SmithingMenu`.
pub struct SmithingMenu {
    behavior: MenuBehavior,
    result_container: SyncSmithingResultContainer,
    block_pos: BlockPos,
    inventory: SyncPlayerInv,
}

impl SmithingMenu {
    #[must_use]
    pub fn new(inventory: SyncPlayerInv, container_id: u8, block_pos: BlockPos) -> Self {
        let mut menu_slots = Vec::with_capacity(slots::TOTAL_SLOTS);

        let result_container: SyncSmithingResultContainer =
            Arc::new(SyncMutex::new(ResultContainer::new()));

        // Slot 0: Template (trim template or upgrade template)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 0)));

        // Slot 1: Base (armor piece or tool)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 1)));

        // Slot 2: Addition (trim material or netherite ingot)
        menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), 2)));

        // Slot 3: Output
        menu_slots.push(SlotType::Result(ResultSlot::new(result_container.clone())));

        // Slots 4-37: Standard inventory (34 slots for 27 inv + 9 hotbar)
        for i in 0..34 {
            menu_slots.push(SlotType::Normal(NormalSlot::new(inventory.clone(), i + 3)));
        }

        Self {
            behavior: MenuBehavior::new(
                menu_slots,
                container_id,
                Some(&vanilla_menu_types::SMITHING),
            ),
            result_container,
            block_pos,
            inventory,
        }
    }

    #[must_use]
    pub fn menu_type() -> MenuTypeRef {
        &vanilla_menu_types::SMITHING
    }

    #[must_use]
    pub const fn block_pos(&self) -> BlockPos {
        self.block_pos
    }

    fn update_result(&self, guard: &mut ContainerLockGuard) {
        let player_inv_refs = self.behavior.slots[slots::TEMPLATE].all_container_refs();
        let inventory = player_inv_refs.iter()
            .find_map(|r| {
                if let ContainerRef::PlayerInventory(inv) = r {
                    Some(inv.clone())
                } else {
                    None
                }
            });

        let inventory = match inventory {
           Some(inv) => inv,
            None => return,
        };
        let player_inv_id = ContainerId::from_arc(&inventory);

        let template = guard.get(player_inv_id)
            .expect("player inventory not locked")
            .get_item(slots::TEMPLATE);

        let base = guard.get(player_inv_id)
            .expect("player inventory not locked")
            .get_item(slots::BASE);

        let addition = guard.get(player_inv_id)
            .expect("player inventory not locked")
            .get_item(slots::ADDITION);

        let result_stack = if template.is_empty() || base.is_empty() || addition.is_empty() {
            ItemStack::empty()
        } else if let Some(recipe) = REGISTRY.recipes.find_smithing_recipe(template, base, addition) {
            recipe.assemble(base, addition).unwrap_or_else(ItemStack::empty)
        } else {
            ItemStack::empty()
        };

        guard.get_mut(ContainerId::from_arc(&self.result_container))
            .expect("result container not locked")
            .set_item(0, result_stack);
    }
}

impl Menu for SmithingMenu {
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

        // Handle output slot being taken
        if slot_index == slots::OUTPUT {
            let clicked = stack.clone();

            // Consume inputs (1 each)
            let player_inv_id = ContainerId::from_arc(&self.inventory);

            // Decrement template
            let template_slot = guard.get_mut(player_inv_id)
                .expect("player inventory not locked")
                .get_item_mut(slots::TEMPLATE);
            template_slot.count = (template_slot.count - 1).max(0);
            if template_slot.count == 0 {
                *template_slot = ItemStack::empty();
            }

            // Decrement base
            let base_slot = guard.get_mut(player_inv_id)
                .expect("player inventory not locked")
                .get_item_mut(slots::BASE);
            base_slot.count = (base_slot.count - 1).max(0);
            if base_slot.count == 0 {
                *base_slot = ItemStack::empty();
            }

            // Decrement addition
            let addition_slot = guard.get_mut(player_inv_id)
                .expect("player inventory not locked")
                .get_item_mut(slots::ADDITION);
            addition_slot.count = (addition_slot.count - 1).max(0);
            if addition_slot.count == 0 {
                *addition_slot = ItemStack::empty();
            }

            // Re-check recipe after consuming
            self.update_result(guard);

            return clicked;
        }

        let clicked = stack.clone();
        let mut stack_mut = stack;

        let smithing_slots = 4;
        let total_slots = self.behavior.slots.len();

        let moved = if slot_index < smithing_slots {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, smithing_slots, total_slots, true)
        } else {
            self.behavior.move_item_stack_to(guard, &mut stack_mut, 0, smithing_slots, false)
        };

        if !moved {
            return ItemStack::empty();
        }

        self.behavior.slots[slot_index].set_item(guard, stack_mut.clone());

        // Update result when any input changes
        if slot_index < slots::ADDITION {
            self.update_result(guard);
        }

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

impl MenuInstance for SmithingMenu {
    fn menu_type(&self) -> MenuTypeRef {
        &vanilla_menu_types::SMITHING
    }

    fn container_id(&self) -> u8 {
        self.behavior.container_id
    }
}

/// Provider for creating smithing menus.
pub struct SmithingMenuProvider {
    inventory: SyncPlayerInv,
    pos: BlockPos,
}

impl SmithingMenuProvider {
    #[must_use]
    pub const fn new(inventory: SyncPlayerInv, pos: BlockPos) -> Self {
        Self { inventory, pos }
    }
}

impl MenuProvider for SmithingMenuProvider {
    fn title(&self) -> TextComponent {
        translations::CONTAINER_UPGRADE.msg().into()
    }

    fn create(&self, container_id: u8) -> Box<dyn MenuInstance> {
        Box::new(SmithingMenu::new(self.inventory.clone(), container_id, self.pos))
    }
}