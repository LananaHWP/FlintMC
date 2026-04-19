//! Smelting recipe types (furnace, blast furnace, smoker, campfire).

use steel_utils::Identifier;

use super::Ingredient;
use crate::{item_stack::ItemStack, items::ItemRef};

/// Type of smeltingrecipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmeltingRecipeType {
    /// Regular furnace (200 ticks).
    Furnace,
    /// Blast furnace (100 ticks).
    BlastFurnace,
    /// Smoker (100 ticks).
    Smoker,
    /// Campfire (600 ticks).
    Campfire,
}

impl SmeltingRecipeType {
    /// Returns the cooking time in ticks for this recipe type.
    #[must_use]
    pub const fn cooking_time(self) -> u32 {
        match self {
            Self::Furnace => 200,
            Self::BlastFurnace => 100,
            Self::Smoker => 100,
            Self::Campfire => 600,
        }
    }
}

/// A smelting recipe (furnace, blast furnace, smoker, or campfire).
#[derive(Debug, Clone)]
pub struct SmeltingRecipe {
    /// Recipe identifier.
    pub id: Identifier,
    /// Type of smelting recipe.
    pub recipe_type: SmeltingRecipeType,
    /// Input ingredient.
    pub ingredient: Ingredient,
    /// Result item.
    pub result: ItemRef,
    /// Cooking time in ticks (may differ from default for this recipe type).
    pub cooking_time: u32,
    /// Experience awarded.
    pub experience: f32,
}

impl SmeltingRecipe {
    /// Creates a new smelting recipe.
    #[must_use]
    pub const fn new(
        id: Identifier,
        recipe_type: SmeltingRecipeType,
        ingredient: Ingredient,
        result: ItemRef,
        experience: f32,
    ) -> Self {
        Self {
            id,
            recipe_type,
            ingredient,
            result,
            cooking_time: recipe_type.cooking_time(),
            experience,
        }
    }

    /// Creates a new smelting recipe with custom cooking time.
    #[must_use]
    pub const fn with_cooking_time(
        id: Identifier,
        recipe_type: SmeltingRecipeType,
        ingredient: Ingredient,
        result: ItemRef,
        cooking_time: u32,
        experience: f32,
    ) -> Self {
        Self {
            id,
            recipe_type,
            ingredient,
            result,
            cooking_time,
            experience,
        }
    }

    /// Tests if the given item stack matches the ingredient.
    #[must_use]
    pub fn matches(&self, input: &ItemStack) -> bool {
        self.ingredient.test(input)
    }

    /// Applies this recipe to the input item stack, returning the result.
    #[must_use]
    pub fn apply(&self, _input: &ItemStack) -> Option<ItemStack> {
        Some(ItemStack::new(self.result))
    }
}