//! Stonecutter recipe type.

use steel_utils::Identifier;

use crate::{item_stack::ItemStack, items::ItemRef};

use super::crafting::RecipeResult;
use super::ingredient::Ingredient;

#[derive(Debug)]
pub struct StonecutterRecipe {
    pub id: Identifier,
    pub ingredient: Ingredient,
    pub result: RecipeResult,
}

impl StonecutterRecipe {
    #[must_use]
    pub fn new(id: Identifier, ingredient: Ingredient, result: ItemRef, count: i32) -> Self {
        Self {
            id,
            ingredient,
            result: RecipeResult { item: result, count },
        }
    }

    #[must_use]
    pub fn test(&self, stack: &ItemStack) -> bool {
        self.ingredient.test(stack)
    }

    #[must_use]
    pub fn assemble(&self) -> ItemStack {
        self.result.to_item_stack()
    }
}