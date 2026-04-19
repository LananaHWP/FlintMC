//! Recipe system for crafting and other recipe types.
//!
//! This module provides the data structures and matching logic for Minecraft recipes.
//! Currently supports crafting recipes (shaped and shapeless) and stonecutting recipes.

mod crafting;
mod ingredient;
mod registry;
mod smithing;
mod smelting;
mod stonecutter;

pub use crafting::{
    CraftingCategory, CraftingInput, CraftingRecipe, PositionedCraftingInput, RecipeResult,
    ShapedRecipe, ShapelessRecipe,
};
pub use ingredient::Ingredient;
pub use registry::{RecipeRegistry, SmithingRecipeResult};
pub use smithing::{SmithingTransformRecipe, SmithingTrimRecipe};
pub use smelting::{SmeltingRecipe, SmeltingRecipeType};
pub use stonecutter::StonecutterRecipe;
