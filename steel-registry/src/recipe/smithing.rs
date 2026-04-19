//! Smithing recipe types (trim and transform).

use steel_utils::{codec::VarInt, serial::WriteTo, Identifier};

use crate::{items::ItemRef, trim_material::TrimMaterialRef, trim_pattern::TrimPatternRef, RegistryEntry, RegistryExt, REGISTRY};

use super::ingredient::Ingredient;
use crate::item_stack::ItemStack;

#[derive(Debug)]
pub struct SmithingTrimRecipe {
    pub id: Identifier,
    pub template: Ingredient,
    pub base: Ingredient,
    pub addition: Ingredient,
    pub pattern: TrimPatternRef,
}

impl SmithingTrimRecipe {
    #[must_use]
    pub fn new(
        id: Identifier,
        template: Ingredient,
        base: Ingredient,
        addition: Ingredient,
        pattern: TrimPatternRef,
    ) -> Self {
        Self {
            id,
            template,
            base,
            addition,
            pattern,
        }
    }

    #[must_use]
    pub fn matches(&self, template_stack: &ItemStack, base_stack: &ItemStack, addition_stack: &ItemStack) -> bool {
        self.template.test(template_stack)
            && self.base.test(base_stack)
            && self.addition.test(addition_stack)
    }

    #[must_use]
    pub fn assemble(&self, base_stack: &ItemStack, addition_stack: &ItemStack) -> Option<ItemStack> {
        if base_stack.is_empty() || addition_stack.is_empty() {
            return None;
        }

        let material = self.get_trim_material(addition_stack)?;

        let mut result = base_stack.clone();
        result.count = 1;

        let mut trim_bytes = Vec::new();
        let pattern_id = self.pattern.key.to_string();
        let material_id = material.key.to_string();

        let pattern_len = VarInt(pattern_id.len() as i32);
        let material_len = VarInt(material_id.len() as i32);
        pattern_len.write(&mut trim_bytes).unwrap();
        trim_bytes.extend_from_slice(pattern_id.as_bytes());
        material_len.write(&mut trim_bytes).unwrap();
        trim_bytes.extend_from_slice(material_id.as_bytes());

        use crate::data_components::vanilla_components::TRIM;
        result
            .patch_mut()
            .set_raw(TRIM.key.clone(), crate::data_components::ComponentData::Other(trim_bytes));

        Some(result)
    }

    fn get_trim_material(&self, addition_stack: &ItemStack) -> Option<TrimMaterialRef> {
        let item = addition_stack.item;
        let key = item.key();
        REGISTRY.trim_materials.by_key(key)
    }
}

#[derive(Debug)]
pub struct SmithingTransformRecipe {
    pub id: Identifier,
    pub template: Ingredient,
    pub base: Ingredient,
    pub addition: Ingredient,
    pub result: ItemRef,
}

impl SmithingTransformRecipe {
    #[must_use]
    pub fn new(
        id: Identifier,
        template: Ingredient,
        base: Ingredient,
        addition: Ingredient,
        result: ItemRef,
    ) -> Self {
        Self {
            id,
            template,
            base,
            addition,
            result,
        }
    }

    #[must_use]
    pub fn matches(
        &self,
        template_stack: &ItemStack,
        base_stack: &ItemStack,
        addition_stack: &ItemStack,
    ) -> bool {
        self.template.test(template_stack)
            && self.base.test(base_stack)
            && self.addition.test(addition_stack)
    }

    #[must_use]
    pub fn assemble(&self) -> ItemStack {
        ItemStack::new(self.result)
    }
}