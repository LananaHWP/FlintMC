use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::enchantment::{Enchantment, EnchantmentRef};
use steel_registry::{REGISTRY, RegistryExt, TaggedRegistryExt};
use rand::RngExt;

pub struct EnchantmentEffectResult {
    pub damage_bonus: f32,
    pub armor_bonus: f32,
    pub armor_toughness_bonus: f32,
    pub knockback_resistance: f32,
    pub mining_speed_multiplier: f32,
    pub luck_bonus: i32,
    pub spawn_chance_bonus: f32,
    pub fire_aspect_ticks: i32,
    pub projectile_damage: f32,
}

impl Default for EnchantmentEffectResult {
    fn default() -> Self {
        Self {
            damage_bonus: 0.0,
            armor_bonus: 0.0,
            armor_toughness_bonus: 0.0,
            knockback_resistance: 0.0,
            mining_speed_multiplier: 1.0,
            luck_bonus: 0,
            spawn_chance_bonus: 0.0,
            fire_aspect_ticks: 0,
            projectile_damage: 0.0,
        }
    }
}

impl EnchantmentEffectResult {
    pub fn from_item(item: &ItemStack) -> Self {
        if item.is_empty() {
            return Self::default();
        }

        let Some(enchantments) = item.get_enchantments() else {
            return Self::default();
        };

        let mut result = Self::default();

        for (enchant_key, &level) in enchantments.iter() {
            let Some(enchant) = REGISTRY.enchantments.by_key(enchant_key) else {
                continue;
            };

            Self::apply_enchantment_effect(enchant, level as i32, &mut result);
        }

        result
    }

    fn apply_enchantment_effect(enchant: EnchantmentRef, level: i32, result: &mut Self) {
        match &*enchant.key.path {
            "sharpness" => {
                let base = if level <= 0 {
                    0.0
                } else {
                    1.0 + (level as f32 - 1.0) * 0.5
                };
                result.damage_bonus += base;
            }
            "smite" => {
                let base = if level <= 0 { 0.0 } else { 2.5 * level as f32 };
                result.damage_bonus += base;
            }
            "bane_of_arthropods" => {
                let base = if level <= 0 { 0.0 } else { 2.5 * level as f32 };
                result.damage_bonus += base;
            }
            "power" => {
                let base = if level <= 0 { 0.0 } else { 1.25 * level as f32 };
                result.projectile_damage += base;
            }
            "punch" => {
                result.damage_bonus += level as f32 * 3.0;
            }
            "knockback" => {
                result.damage_bonus += level as f32 * 3.0;
            }
            "fire_aspect" => {
                result.fire_aspect_ticks = level * 4;
            }
            "looting" => {
                result.luck_bonus += level;
            }
            "fortune" => {
                result.luck_bonus += level * 3;
            }
            "efficiency" => {
                if level > 0 {
                    let base = 1.0 + (level as f32 - 1.0) * 0.5;
                    result.mining_speed_multiplier *= base;
                }
            }
            "silk_touch" => {
                if level > 0 {
                    result.spawn_chance_bonus = 1.0;
                }
            }
            "protection" | "fire_protection" | "blast_protection" | "projectile_protection" => {
                let base = if level <= 0 { 0.0 } else { level as f32 };
                result.armor_bonus += base;
            }
            "feather_falling" => {
                let base = if level <= 0 { 0.0 } else { level as f32 };
                result.armor_bonus += base;
            }
            "thorns" => {
                if level > 0 {
                    result.armor_bonus += level as f32 * 0.5;
                }
            }
            "depth_strider" | "frost_walker" | "respiration" | "aqua_affinity" => {}
            "unbreaking" => {}
            "mending" | "vanishing_curse" | "binding_curse" => {}
            "flame" | "infinity" => {}
            "luck_of_the_sea" | "loyalty" | "riptide" | "channeling" => {}
            "multishot" | "quick_charge" | "piercing" => {}
            _ => {}
        }
    }
}

pub struct MiningSpeedResult {
    pub speed: f32,
    pub can_mine: bool,
}

impl MiningSpeedResult {
    pub fn calculate(item: &ItemStack, base_speed: f32, can_harvest: bool) -> Self {
        if item.is_empty() {
            return Self {
                speed: base_speed,
                can_mine: can_harvest,
            };
        }

        let mut speed = base_speed;
        let mut can_mine = can_harvest;

        let Some(enchantments) = item.get_enchantments() else {
            return Self {
                speed,
                can_mine,
            };
        };

        for (enchant_key, &level) in enchantments.iter() {
            if &*enchant_key.path == "efficiency" && level > 0 {
                speed += level as f32 * base_speed * 0.3;
            }
            if &*enchant_key.path == "silk_touch" && level > 0 && !can_harvest {
                can_mine = false;
                speed = 0.0;
            }
        }

        Self {
            speed,
            can_mine,
        }
    }
}

pub struct DamageReductionResult {
    pub damage: f32,
    pub armor_to_remove: f32,
    pub unbreaking_damage: i32,
}

impl DamageReductionResult {
    pub fn calculate(damage: f32, armor: f32, armor_toughness: f32, unbreaking_level: i32) -> Self {
        let reduction = armor.min(damage);
        let remaining = damage - reduction;
        let toughness_reduction = (remaining - armor_toughness).max(0.0);
        let final_damage = (remaining - toughness_reduction).max(0.0);

        let unbreaking_damage =
            if unbreaking_level > 0 && rand::rng().random_range(0..unbreaking_level + 1) == 0 {
                1
            } else {
                0
            };

        Self {
            damage: final_damage,
            armor_to_remove: reduction,
            unbreaking_damage,
        }
    }
}

pub fn is_enchantable(item: &ItemStack) -> bool {
    if item.is_empty() {
        return false;
    }
    REGISTRY
        .items
        .is_in_tag(item.item(), &steel_utils::Identifier::vanilla_static("enchantable"))
}

pub struct EnchantmentOption {
    pub enchantment: EnchantmentRef,
    pub level: i32,
    pub cost: i32,
}

pub fn get_enchantment_options(item: &ItemStack, xp_level: i32) -> Vec<EnchantmentOption> {
    if item.is_empty() {
        return Vec::new();
    }

    let item_ref = item.item();

    let Some(tag) = item.get_tool() else {
        return Vec::new();
    };

    let enchantability = get_item_enchantability(item_ref);
    let effective_level = (xp_level / 2).max(1).min(30).min(xp_level);

    if effective_level < 1 {
        return Vec::new();
    }

    let mut options = Vec::new();
    let mut rng = rand::rng();

    for (_, enchant) in REGISTRY.enchantments.iter() {
        if !enchant.can_enchant(item_ref) {
            continue;
        }

        if !Enchantment::is_compatible_with_existing(enchant, item) {
            continue;
        }

        let min_cost = enchant.min_cost.base + enchant.min_cost.per_level_above_first * (effective_level - 1);
        let max_cost = enchant.max_cost.base + enchant.max_cost.per_level_above_first * (effective_level - 1);

        if min_cost <= 0 {
            continue;
        }

        let cost_roll = rng.random_range(0..50);
        if cost_roll >= min_cost {
            continue;
        }

        let level = calculate_enchantment_level(enchant, effective_level, &mut rng);
        let cost = calculate_cost_for_option(min_cost, max_cost, effective_level);

        options.push(EnchantmentOption {
            enchantment: enchant,
            level,
            cost,
        });
    }

    options.sort_by(|a, b| b.cost.cmp(&a.cost));
    options.truncate(3);
    options
}

fn get_item_enchantability(item: ItemRef) -> i32 {
    use steel_registry::vanilla_items;
    match item {
        item if item == &vanilla_items::ITEMS.iron_sword => 14,
        item if item == &vanilla_items::ITEMS.golden_sword => 22,
        item if item == &vanilla_items::ITEMS.diamond_sword => 10,
        item if item == &vanilla_items::ITEMS.netherite_sword => 15,
        item if item == &vanilla_items::ITEMS.iron_pickaxe => 10,
        item if item == &vanilla_items::ITEMS.golden_pickaxe => 22,
        item if item == &vanilla_items::ITEMS.diamond_pickaxe => 10,
        item if item == &vanilla_items::ITEMS.netherite_pickaxe => 15,
        item if item == &vanilla_items::ITEMS.iron_axe => 10,
        item if item == &vanilla_items::ITEMS.golden_axe => 22,
        item if item == &vanilla_items::ITEMS.diamond_axe => 10,
        item if item == &vanilla_items::ITEMS.netherite_axe => 15,
        item if item == &vanilla_items::ITEMS.iron_helmet => 10,
        item if item == &vanilla_items::ITEMS.golden_helmet => 22,
        item if item == &vanilla_items::ITEMS.diamond_helmet => 10,
        item if item == &vanilla_items::ITEMS.netherite_helmet => 15,
        item if item == &vanilla_items::ITEMS.iron_chestplate => 10,
        item if item == &vanilla_items::ITEMS.golden_chestplate => 22,
        item if item == &vanilla_items::ITEMS.diamond_chestplate => 10,
        item if item == &vanilla_items::ITEMS.netherite_chestplate => 15,
        item if item == &vanilla_items::ITEMS.iron_leggings => 10,
        item if item == &vanilla_items::ITEMS.golden_leggings => 22,
        item if item == &vanilla_items::ITEMS.diamond_leggings => 10,
        item if item == &vanilla_items::ITEMS.netherite_leggings => 15,
        item if item == &vanilla_items::ITEMS.iron_boots => 10,
        item if item == &vanilla_items::ITEMS.golden_boots => 22,
        item if item == &vanilla_items::ITEMS.diamond_boots => 10,
        item if item == &vanilla_items::ITEMS.netherite_boots => 15,
        item if item == &vanilla_items::ITEMS.bow => 10,
        item if item == &vanilla_items::ITEMS.crossbow => 10,
        item if item == &vanilla_items::ITEMS.trident => 8,
        item if item == &vanilla_items::ITEMS.book => 1,
        item if item == &vanilla_items::ITEMS.fishing_rod => 1,
        _ => 0,
    }
}

fn calculate_enchantment_level<R: rand::Rng>(
    enchant: EnchantmentRef,
    xp_level: i32,
    rng: &mut R,
) -> i32 {
    let diff = enchant.max_cost.base - enchant.min_cost.base;
    if diff <= 0 {
        return 1;
    }

    let range = (xp_level * 2).max(1);
    let bonus = rng.random_range(0..range);

    let level = enchant.min_cost.base + (diff * bonus) / range;
    level.max(1).min(enchant.max_level as i32)
}

fn calculate_cost_for_option(min_cost: i32, max_cost: i32, xp_level: i32) -> i32 {
    let mid = (min_cost + max_cost) / 2;
    let adjusted = mid.saturating_sub((xp_level - 1) * 3);
    (adjusted / 2).max(1).min(xp_level)
}