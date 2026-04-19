//! Inventory and container management system.
//!
//! This module provides the core inventory system including containers,
//! menus, crafting, equipment, and recipes.

pub mod anvil_menu;
pub mod beacon_menu;
pub mod brewing_stand_menu;
pub mod cartography_menu;
pub mod chest_menu;
pub mod container;
pub mod crafting;
pub mod crafting_menu;
pub mod enchantment_menu;
pub mod equipment;
pub mod furnace_menu;
pub mod grindstone_menu;
pub mod hopper_menu;
pub mod inventory_menu;
pub mod lectern_menu;
pub mod lock;
pub mod loom_menu;
pub mod menu;
pub mod menu_provider;
pub mod merchant_menu;
pub mod recipe_manager;
pub mod shulker_box_menu;
pub mod slot;
pub mod smithing_menu;
pub mod smoker_menu;
pub mod stonecutter_menu;

pub use anvil_menu::{AnvilMenu, AnvilMenuProvider};
pub use beacon_menu::{BeaconMenu, BeaconMenuProvider};
pub use brewing_stand_menu::{BrewingStandMenu, BrewingStandMenuProvider};
pub use cartography_menu::{CartographyMenu, CartographyMenuProvider};
pub use chest_menu::{ChestMenu, ChestMenuProvider};
pub use crafting_menu::{CraftingMenu, CraftingMenuProvider};
pub use enchantment_menu::{EnchantmentMenu, EnchantmentMenuProvider};
pub use furnace_menu::{FurnaceMenu, FurnaceMenuProvider, FurnaceMenuStandard, FurnaceMenuProviderStandard};
pub use grindstone_menu::{GrindstoneMenu, GrindstoneMenuProvider};
pub use hopper_menu::{HopperMenu, HopperMenuProvider};
pub use lectern_menu::{LecternMenu, LecternMenuProvider};
pub use loom_menu::{LoomMenu, LoomMenuProvider};
pub use lock::{ContainerId, SyncPlayerInv};
pub use menu_provider::{MenuInstance, MenuProvider};
pub use merchant_menu::{MerchantMenu, MerchantMenuProvider};
pub use shulker_box_menu::{ShulkerBoxMenu, ShulkerBoxMenuProvider};
pub use smithing_menu::{SmithingMenu, SmithingMenuProvider};
pub use smoker_menu::{SmokerMenuStandard, SmokerMenuProviderStandard};
pub use stonecutter_menu::{StonecutterMenu, StonecutterMenuProvider};
