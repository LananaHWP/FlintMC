//! Recipe registry for looking up recipes.

use rustc_hash::FxHashMap;
use steel_utils::Identifier;

use super::crafting::{CraftingInput, CraftingRecipe, ShapedRecipe, ShapelessRecipe};
use super::smithing::{SmithingTransformRecipe, SmithingTrimRecipe};
use super::smelting::{SmeltingRecipe, SmeltingRecipeType};
use super::stonecutter::StonecutterRecipe;
use crate::item_stack::ItemStack;

/// Result of matching a smithing recipe.
#[derive(Debug)]
pub enum SmithingRecipeResult<'a> {
    Trim(&'a super::smithing::SmithingTrimRecipe),
    Transform(&'a super::smithing::SmithingTransformRecipe),
}

impl<'a> SmithingRecipeResult<'a> {
    /// Assembles the result item stack.
    pub fn assemble(&self, base: &ItemStack, addition: &ItemStack) -> Option<ItemStack> {
        match self {
            Self::Trim(r) => r.assemble(base, addition),
            Self::Transform(r) => Some(r.assemble()),
        }
    }
}

/// Registry for all recipes.
pub struct RecipeRegistry {
    /// All recipes in registration order (unified storage for RegistryExt).
    recipes_by_id: Vec<&'static CraftingRecipe>,
    /// Map from recipe key to index in `recipes_by_id`.
    recipes_by_key: FxHashMap<Identifier, usize>,
    /// All shaped crafting recipes (for type-specific iteration).
    shaped_recipes: Vec<&'static ShapedRecipe>,
    /// All shapeless crafting recipes (for type-specific iteration).
    shapeless_recipes: Vec<&'static ShapelessRecipe>,
    /// All stonecutter recipes.
    stonecutter_recipes: Vec<&'static StonecutterRecipe>,
    /// All smithing trim recipes.
    smithing_trim_recipes: Vec<&'static SmithingTrimRecipe>,
    /// All smithing transform recipes.
    smithing_transform_recipes: Vec<&'static SmithingTransformRecipe>,
    /// All furnace recipes.
    furnace_recipes: Vec<&'static SmeltingRecipe>,
    /// All blast furnace recipes.
    blast_furnace_recipes: Vec<&'static SmeltingRecipe>,
    /// All smoker recipes.
    smoker_recipes: Vec<&'static SmeltingRecipe>,
    /// All campfire recipes.
    campfire_recipes: Vec<&'static SmeltingRecipe>,
    /// Whether registration is still allowed.
    allows_registering: bool,
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeRegistry {
    /// Creates a new empty recipe registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            recipes_by_id: Vec::new(),
            recipes_by_key: FxHashMap::default(),
            shaped_recipes: Vec::new(),
            shapeless_recipes: Vec::new(),
            stonecutter_recipes: Vec::new(),
            smithing_trim_recipes: Vec::new(),
            smithing_transform_recipes: Vec::new(),
            furnace_recipes: Vec::new(),
            blast_furnace_recipes: Vec::new(),
            smoker_recipes: Vec::new(),
            campfire_recipes: Vec::new(),
            allows_registering: true,
        }
    }

    /// Registers a shaped recipe.
    pub fn register_shaped(&mut self, recipe: &'static ShapedRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        let id = self.recipes_by_id.len();
        self.recipes_by_key.insert(recipe.id.clone(), id);
        self.recipes_by_id
            .push(Box::leak(Box::new(CraftingRecipe::Shaped(recipe))));
        self.shaped_recipes.push(recipe);
    }

    /// Registers a shapeless recipe.
    pub fn register_shapeless(&mut self, recipe: &'static ShapelessRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        let id = self.recipes_by_id.len();
        self.recipes_by_key.insert(recipe.id.clone(), id);
        self.recipes_by_id
            .push(Box::leak(Box::new(CraftingRecipe::Shapeless(recipe))));
        self.shapeless_recipes.push(recipe);
    }

    /// Registers a stonecutter recipe.
    pub fn register_stonecutter(&mut self, recipe: &'static StonecutterRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        let id = self.recipes_by_id.len();
        self.recipes_by_key.insert(recipe.id.clone(), id);
        self.recipes_by_id
            .push(Box::leak(Box::new(CraftingRecipe::Stonecutter(recipe))));
        self.stonecutter_recipes.push(recipe);
    }

    /// Registers a smithing trim recipe.
    pub fn register_smithing_trim(&mut self, recipe: &'static SmithingTrimRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        self.smithing_trim_recipes.push(recipe);
    }

    /// Registers a smithing transform recipe.
    pub fn register_smithing_transform(&mut self, recipe: &'static SmithingTransformRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        self.smithing_transform_recipes.push(recipe);
    }

    /// Registers a furnace smelting recipe.
    pub fn register_furnace(&mut self, recipe: &'static SmeltingRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        self.furnace_recipes.push(recipe);
    }

    /// Registers a blast furnace smelting recipe.
    pub fn register_blast_furnace(&mut self, recipe: &'static SmeltingRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        self.blast_furnace_recipes.push(recipe);
    }

    /// Registers a smoker smelting recipe.
    pub fn register_smoker(&mut self, recipe: &'static SmeltingRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        self.smoker_recipes.push(recipe);
    }

    /// Registers a campfire smelting recipe.
    pub fn register_campfire(&mut self, recipe: &'static SmeltingRecipe) {
        assert!(
            self.allows_registering,
            "Cannot register recipes after the registry has been frozen"
        );
        self.campfire_recipes.push(recipe);
    }

    /// Finds a matching smelting recipe for the given input and recipe type.
    #[must_use]
    pub fn find_smelting_recipe(
        &self,
        input: &ItemStack,
        recipe_type: SmeltingRecipeType,
    ) -> Option<&'static SmeltingRecipe> {
        let recipes = match recipe_type {
            SmeltingRecipeType::Furnace => &self.furnace_recipes,
            SmeltingRecipeType::BlastFurnace => &self.blast_furnace_recipes,
            SmeltingRecipeType::Smoker => &self.smoker_recipes,
            SmeltingRecipeType::Campfire => &self.campfire_recipes,
        };

        for recipe in recipes {
            if recipe.matches(input) {
                return Some(recipe);
            }
        }
        None
    }

    /// Finds a matching campfire recipe for the given input.
    #[must_use]
    pub fn find_campfire_recipe(&self, input: &ItemStack) -> Option<&'static SmeltingRecipe> {
        self.find_smelting_recipe(input, SmeltingRecipeType::Campfire)
    }

    /// Finds a matching furnace recipe for the given input.
    #[must_use]
    pub fn find_furnace_recipe(&self, input: &ItemStack) -> Option<&'static SmeltingRecipe> {
        self.find_smelting_recipe(input, SmeltingRecipeType::Furnace)
    }

    /// Finds a matching smithing (trim or transform) recipe for the given inputs.
    /// Returns trim result, then transform result, or None.
    #[must_use]
    pub fn find_smithing_recipe(
        &self,
        template: &ItemStack,
        base: &ItemStack,
        addition: &ItemStack,
    ) -> Option<SmithingRecipeResult> {
        for recipe in &self.smithing_trim_recipes {
            if recipe.matches(template, base, addition) {
                return Some(SmithingRecipeResult::Trim(recipe));
            }
        }

        for recipe in &self.smithing_transform_recipes {
            if recipe.matches(template, base, addition) {
                return Some(SmithingRecipeResult::Transform(recipe));
            }
        }

        None
    }

    /// Finds a matching stonecutter recipe for the given item stack.
    #[must_use]
    pub fn find_stonecutter_recipe(&self, input: &ItemStack) -> Option<&'static StonecutterRecipe> {
        for recipe in &self.stonecutter_recipes {
            if recipe.test(input) {
                return Some(recipe);
            }
        }
        None
    }

    /// Finds a matching crafting recipe for the given positioned input.
    /// Returns the first matching recipe, or None if no recipe matches.
    #[must_use]
    pub fn find_crafting_recipe(&self, input: &CraftingInput) -> Option<CraftingRecipe> {
        // Try shaped recipes first (they're more specific)
        for recipe in &self.shaped_recipes {
            if recipe.matches(input) {
                return Some(CraftingRecipe::Shaped(recipe));
            }
        }

        // Then try shapeless
        for recipe in &self.shapeless_recipes {
            if recipe.matches(input) {
                return Some(CraftingRecipe::Shapeless(recipe));
            }
        }

        None
    }

    /// Finds a matching crafting recipe for a 2x2 grid.
    /// Only checks recipes that can fit in a 2x2 grid.
    #[must_use]
    pub fn find_crafting_recipe_2x2(&self, input: &CraftingInput) -> Option<CraftingRecipe> {
        // Try shaped recipes first (they're more specific)
        for recipe in &self.shaped_recipes {
            if recipe.fits_in_2x2() && recipe.matches(input) {
                return Some(CraftingRecipe::Shaped(recipe));
            }
        }

        // Then try shapeless
        for recipe in &self.shapeless_recipes {
            if recipe.fits_in_2x2() && recipe.matches(input) {
                return Some(CraftingRecipe::Shapeless(recipe));
            }
        }

        None
    }

    /// Gets a shaped recipe by its identifier.
    #[must_use]
    pub fn get_shaped(&self, id: &Identifier) -> Option<&'static ShapedRecipe> {
        self.shaped_recipes.iter().find(|r| &r.id == id).copied()
    }

    /// Gets a shapeless recipe by its identifier.
    #[must_use]
    pub fn get_shapeless(&self, id: &Identifier) -> Option<&'static ShapelessRecipe> {
        self.shapeless_recipes.iter().find(|r| &r.id == id).copied()
    }

    /// Gets a stonecutter recipe by its identifier.
    #[must_use]
    pub fn get_stonecutter(&self, id: &Identifier) -> Option<&'static StonecutterRecipe> {
        self.stonecutter_recipes.iter().find(|r| &r.id == id).copied()
    }

    /// Returns the number of shaped recipes.
    #[must_use]
    pub fn shaped_count(&self) -> usize {
        self.shaped_recipes.len()
    }

    /// Returns the number of shapeless recipes.
    #[must_use]
    pub fn shapeless_count(&self) -> usize {
        self.shapeless_recipes.len()
    }

    /// Iterates over all shaped recipes.
    pub fn iter_shaped(&self) -> impl Iterator<Item = &'static ShapedRecipe> + '_ {
        self.shaped_recipes.iter().copied()
    }

    /// Iterates over all shapeless recipes.
    pub fn iter_shapeless(&self) -> impl Iterator<Item = &'static ShapelessRecipe> + '_ {
        self.shapeless_recipes.iter().copied()
    }

    /// Returns the number of stonecutter recipes.
    #[must_use]
    pub fn stonecutter_count(&self) -> usize {
        self.stonecutter_recipes.len()
    }

    /// Iterates over all stonecutter recipes.
    pub fn iter_stonecutter(&self) -> impl Iterator<Item = &'static StonecutterRecipe> + '_ {
        self.stonecutter_recipes.iter().copied()
    }

    /// Gets a campfire recipe by its identifier.
    #[must_use]
    pub fn get_campfire(&self, id: &Identifier) -> Option<&'static SmeltingRecipe> {
        self.campfire_recipes.iter().find(|r| &r.id == id).copied()
    }

    /// Gets a furnace recipe by its identifier.
    #[must_use]
    pub fn get_furnace(&self, id: &Identifier) -> Option<&'static SmeltingRecipe> {
        self.furnace_recipes.iter().find(|r| &r.id == id).copied()
    }

    /// Returns the number of furnace recipes.
    #[must_use]
    pub fn furnace_count(&self) -> usize {
        self.furnace_recipes.len()
    }

    /// Returns the number of campfire recipes.
    #[must_use]
    pub fn campfire_count(&self) -> usize {
        self.campfire_recipes.len()
    }

    /// Iterates over all furnace recipes.
    pub fn iter_furnace(&self) -> impl Iterator<Item = &'static SmeltingRecipe> + '_ {
        self.furnace_recipes.iter().copied()
    }

    /// Iterates over all campfire recipes.
    pub fn iter_campfire(&self) -> impl Iterator<Item = &'static SmeltingRecipe> + '_ {
        self.campfire_recipes.iter().copied()
    }
}

impl crate::RegistryExt for RecipeRegistry {
    type Entry = CraftingRecipe;

    fn freeze(&mut self) {
        self.allows_registering = false;
    }

    fn by_id(&self, id: usize) -> Option<&'static CraftingRecipe> {
        self.recipes_by_id.get(id).copied()
    }

    fn by_key(&self, key: &Identifier) -> Option<&'static CraftingRecipe> {
        self.recipes_by_key
            .get(key)
            .and_then(|&id| self.recipes_by_id.get(id).copied())
    }

    fn id_from_key(&self, key: &Identifier) -> Option<usize> {
        self.recipes_by_key.get(key).copied()
    }

    fn len(&self) -> usize {
        self.recipes_by_id.len()
    }

    fn is_empty(&self) -> bool {
        self.recipes_by_id.is_empty()
    }
}

impl crate::RegistryEntry for CraftingRecipe {
    fn key(&self) -> &Identifier {
        self.id()
    }

    fn try_id(&self) -> Option<usize> {
        use crate::RegistryExt;
        crate::REGISTRY.recipes.id_from_key(self.id())
    }
}
