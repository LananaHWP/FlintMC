use rustc_hash::FxHashMap;
use serde::Deserialize;
use steel_registry::{vanilla_blocks, REGISTRY, RegistryExt};
use steel_utils::Identifier;
use steel_utils::random::xoroshiro::Xoroshiro;
use steel_utils::random::Random;
use steel_utils::BlockStateId;
use std::sync::OnceLock;

use crate::chunk::chunk_access::ChunkAccess;

#[derive(Debug, Clone, Deserialize)]
pub struct PlacedFeatureJson {
    pub feature: String,
    pub placement: Vec<PlacementCondition>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum PlacementCondition {
    #[serde(rename = "minecraft:count")]
    Count(CountPlacement),
    #[serde(rename = "minecraft:in_square")]
    InSquare,
    #[serde(rename = "minecraft:biome")]
    Biome,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CountPlacement {
    pub count: i32,
}

static PLACED_FEATURES: OnceLock<FxHashMap<Identifier, PlacedFeatureJson>> = OnceLock::new();

pub fn get_placed_feature(name: &Identifier) -> Option<&'static PlacedFeatureJson> {
    let map = PLACED_FEATURES.get_or_init(|| {
        let mut m = FxHashMap::default();
        if let Ok(dir) = std::fs::read_dir("steel-registry/build_assets/builtin_datapacks/minecraft/worldgen/placed_feature") {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<PlacedFeatureJson>(&content) {
                            let name = path.file_stem().unwrap().to_str().unwrap();
                            let id = Identifier::vanilla(name.to_string());
                            m.insert(id, json);
                        }
                    }
                }
            }
        }
        m
    });
    map.get(name)
}

pub struct FeatureGenerator {
    seed: u64,
    random: Xoroshiro,
}

impl FeatureGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            random: Xoroshiro::from_seed(seed),
        }
    }

    fn next_int(&mut self, min: i32, max: i32) -> i32 {
        min + (self.random.next_i32() % (max - min))
    }

    pub fn apply_biome_decorations(&mut self, chunk: &ChunkAccess) {
        let pos = chunk.pos();
        let chunk_min_x = pos.0.x * 16;
        let chunk_min_z = pos.0.y * 16;
        let min_y = chunk.min_y();

        let get_biome = |qx: i32, qz: i32| -> u16 {
            let biome_data = chunk.sections().read_all_biomes();
            let section_idx = 0usize.min(chunk.sections().sections.len().saturating_sub(1));
            let local_qx = ((qx & 3) + 4) as usize;
            let local_qz = ((qz & 3) + 4) as usize;
            biome_data.get(section_idx * 64 + local_qz * 4 + local_qx).copied().unwrap_or(0)
        };

        for local_x in 0..16usize {
            for local_z in 0..16usize {
                let world_x = chunk_min_x + local_x as i32;
                let world_z = chunk_min_z + local_z as i32;

                let biome_quart_x = world_x >> 2;
                let biome_quart_z = world_z >> 2;
                let biome_id = get_biome(biome_quart_x, biome_quart_z);

                if let Some(biome) = REGISTRY.biomes.by_id(biome_id as usize) {
                    for stage_idx in 0..biome.features.len() {
                        for feature_id in &biome.features[stage_idx] {
                            self.process_feature(chunk, local_x, local_z, world_x, world_z, feature_id);
                        }
                    }
                }
            }
        }
        chunk.mark_dirty();
    }

    fn process_feature(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, world_x: i32, world_z: i32, feature_id: &Identifier) {
        let ground_y = chunk.get_surface_height(local_x, local_z).unwrap_or(64);

        let feature_path = feature_id.path.as_ref();

        match feature_path {
            // Trees
            "oak" | "oak_checked" | "fancy_oak" | "fancy_oak_bees" | "fancy_oak_bees_0002" | "oak_bees" | "oak_bees_002" | "oak_bees_0002_leaf_litter" | "trees_plains" | "trees_giant" | "trees_giant_spruce" | "trees_giant_jungle" | "trees_birch" | "trees_birch_and_oak_leaf_litter" | "trees_taiga" | "trees_taiga_large" | "trees_dark_forest" | "trees_flower_forest" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "oak");
            }
            "spruce" | "spruce_checked" | "trees_snowy" | "spruce_on_snow" | "pine_on_snow" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "spruce");
            }
            "birch" | "birch_checked" | "birch_bees" | "birch_bees_002" | "birch_bees_0002" | "birch_bees_0002_leaf_litter" | "birch_tall" | "birch_leaf_litter" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "birch");
            }
            "acacia" | "acacia_checked" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "acacia");
            }
            "jungle" | "jungle_tree" | "jungle_bush" | "trees_jungle" | "trees_jungle_edge" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "jungle");
            }
            "dark_oak" | "dark_oak_checked" | "dark_forest_vegetation" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "dark_oak");
            }
            "mangrove" | "mangrove_checked" | "trees_mangrove" | "trees_swamp" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "mangrove");
            }
            "cherry" | "cherry_checked" | "cherry_bees" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "cherry");
            }
            "crimson_fungi" | "crimson_forest_vegetation" | "warped_fungi" | "warped_forest_vegetation" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "crimson");
            }
            "trees_windswept_forest" | "trees_windswept_hills" | "trees_windswept_savanna" | "trees_sparse_jungle" | "trees_water" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "oak");
            }
            "pale_garden_vegetation" | "pale_forest_tree" => {
                self.place_tree_simple(chunk, local_x, local_z, ground_y, "dark_oak");
            }
            // Flowers
            "flower_plain" | "flower_default" | "flower_meadow" | "flower_flower_forest" | "flower_pale_garden" | "flower_plains" | "flower_warm" | "flower_swamp" | "flower_cherry" | "flower_forest_flowers" => {
                self.place_flower_simple(chunk, local_x, local_z, ground_y);
            }
            "wildflowers_meadow" | "wildflowers_birch_forest" => {
                self.place_flower_simple(chunk, local_x, local_z, ground_y);
            }
            "forest_flowers" => {
                self.place_flower_simple(chunk, local_x, local_z, ground_y);
            }
            "pale_garden_flowers" | "pale_forest_flower" => {
                self.place_flower_simple(chunk, local_x, local_z, ground_y);
            }
            "patch_sunflower" => {
                self.place_tall_flowers(chunk, local_x, local_z, ground_y);
            }
            // Grass
            "grass" | "patch_grass_plain" | "patch_grass_normal" | "grass_bonemeal" | "patch_grass_savanna" | "patch_grass_taiga" | "patch_grass_jungle" => {
                self.place_grass_simple(chunk, local_x, local_z, ground_y);
            }
            "patch_tall_grass" | "patch_large_fern" => {
                self.place_tall_grass(chunk, local_x, local_z, ground_y);
            }
            // Mushrooms
            "brown_mushroom_normal" | "red_mushroom_normal" | "brown_mushroom_taiga" | "red_mushroom_taiga" | "brown_mushroom_swamp" | "red_mushroom_swamp" | "brown_mushroom_old_growth" | "red_mushroom_old_growth" | "brown_mushroom_nether" | "red_mushroom_nether" | "mushroom_island_vegetation" => {
                self.place_mushroom_simple(chunk, local_x, local_z, ground_y, "brown");
            }
            // Pumpkins and melons
            "patch_pumpkin" | "pile_pumpkin" => {
                self.place_pumpkin_simple(chunk, local_x, local_z, ground_y);
            }
            "patch_melon" => {
                self.place_melon_simple(chunk, local_x, local_z, ground_y);
            }
            // Sugar cane and reeds
            "patch_sugar_cane" => {
                self.place_sugar_cane_simple(chunk, local_x, local_z, ground_y);
            }
            // Springs
            "spring_water" | "spring_lava" => {
                self.place_spring_simple(chunk, local_x, local_z, ground_y + 4, feature_path);
            }
            "spring_delta" => {
                self.place_spring_simple(chunk, local_x, local_z, ground_y + 4, "spring_lava");
            }
            "spring_lava_frozen" => {
                self.place_spring_simple(chunk, local_x, local_z, ground_y + 4, "spring_lava");
            }
            "spring_closed" | "spring_closed_double" | "spring_open" => {
                self.place_spring_simple(chunk, local_x, local_z, ground_y + 4, "spring_water");
            }
            // Lava lakes
            "lake_lava_underground" | "lake_lava_surface" => {
                self.place_lava_lake_simple(chunk, local_x, local_z, ground_y);
            }
            // Geodes
            "amethyst_geode" => {
                self.place_geode_simple(chunk, local_x, local_z, ground_y - 20);
            }
            // Monster rooms
            "monster_room" | "monster_room_deep" => {
                self.place_monster_room_simple(chunk, local_x, local_z, ground_y - 20);
            }
            // Disks
            "disk_sand" | "disk_gravel" | "disk_clay" | "disk_grass" => {
                self.place_disk_simple(chunk, local_x, local_z, ground_y - 1, feature_path);
            }
            // Snow
            "freeze_top_layer" => {
                self.place_snow_simple(chunk, local_x, local_z, ground_y);
            }
            // Water plants
            "waterlily" => {
                self.place_waterlily_simple(chunk, local_x, local_z, ground_y);
            }
            "seagrass_normal" | "seagrass_river" | "seagrass_swamp" | "seagrass_cold" | "seagrass_deep" | "seagrass_deep_cold" | "seagrass_deep_warm" | "seagrass_warm" => {
                self.place_seagrass(chunk, local_x, local_z, ground_y);
            }
            "sea_pickle" => {
                self.place_sea_pickle_simple(chunk, local_x, local_z, ground_y);
            }
            "kelp" | "kelp_cold" | "kelp_warm" => {
                self.place_kelp(chunk, local_x, local_z, ground_y);
            }
            "warm_ocean_vegetation" => {
                self.place_warm_ocean_vegetation(chunk, local_x, local_z, ground_y);
            }
            // Cave and surface plants
            "glow_lichen" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GLOW_LICHEN);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            "cave_vines" | "cave_vines_plant" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CAVE_VINES);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            "rooted_azalea_tree" => {
                self.place_rooted_azalea(chunk, local_x, local_z, ground_y);
            }
            "bamboo" | "bamboo_light" | "bamboo_vegetation" => {
                self.place_bamboo_simple(chunk, local_x, local_z, ground_y);
            }
            "cactus" => {
                self.place_cactus_simple(chunk, local_x, local_z, ground_y);
            }
            "desert_well" => {
                self.place_desert_well(chunk, local_x, local_z, ground_y);
            }
            "spore_blossom" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPORE_BLOSSOM);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            "moss_patch" => {
                self.place_moss_patch(chunk, local_x, local_z, ground_y);
            }
            "patch_bush" => {
                self.place_bush(chunk, local_x, local_z, ground_y);
            }
            "vines" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::VINE);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            "weeping_vines" => {
                self.place_weeping_vines(chunk, local_x, local_z, ground_y);
            }
            "twisting_vines" => {
                self.place_twisting_vines(chunk, local_x, local_z, ground_y);
            }
            "nether_sprouts" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::NETHER_SPROUTS);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            // Dripstone
            "dripstone_cluster" => {
                self.place_dripstone_cluster(chunk, local_x, local_z, ground_y - 10);
            }
            "pointed_dripstone" => {
                self.place_pointed_dripstone(chunk, local_x, local_z, ground_y - 15);
            }
            // Sculk
            "sculk_vein" => {
                self.place_sculk_vein(chunk, local_x, local_z, ground_y);
            }
            "sculk_patch_deep_dark" | "sculk_patch_ancient_city" => {
                self.place_sculk_patch(chunk, local_x, local_z, ground_y);
            }
            // Ice and snow
            "ice_spike" => {
                self.place_ice_spike(chunk, local_x, local_z, ground_y);
            }
            "ice_patch" => {
                self.place_ice_simple(chunk, local_x, local_z, ground_y);
            }
            "iceberg_packed" | "iceberg_blue" => {
                self.place_iceberg(chunk, local_x, local_z, ground_y);
            }
            "blue_ice" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BLUE_ICE);
                self.place_simple_block(chunk, local_x, local_z, ground_y - 1, block_id);
            }
            // Sweet berries
            "sweet_berry_bush" => {
                self.place_sweet_berry_bush(chunk, local_x, local_z, ground_y);
            }
            // Ferns
            "fern" => {
                self.place_tall_grass(chunk, local_x, local_z, ground_y);
            }
            // Forest rocks
            "forest_rock" => {
                self.place_forest_rock(chunk, local_x, local_z, ground_y);
            }
            // Ores
            "ore_coal_upper" | "ore_coal_lower" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "coal");
            }
            "ore_iron_upper" | "ore_iron_middle" | "ore_iron_small" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "iron");
            }
            "ore_gold" | "ore_gold_extra" | "ore_gold_lower" | "ore_gold_deltas" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "gold");
            }
            "ore_copper" | "ore_copper_large" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "copper");
            }
            "ore_redstone" | "ore_redstone_lower" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "redstone");
            }
            "ore_lapis" | "ore_lapis_buried" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "lapis");
            }
            "ore_diamond" | "ore_diamond_buried" | "ore_diamond_medium" | "ore_diamond_large" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "diamond");
            }
            "ore_emerald" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "emerald");
            }
            "ore_debris_small" | "ore_ancient_debris_large" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "ancient_debris");
            }
            "ore_quartz_nether" | "ore_quartz_deltas" | "ore_gold_nether" | "ore_gold_deltas" | "ore_blackstone" | "ore_gravel_nether" | "ore_magma" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "nether");
            }
            "ore_granite_upper" | "ore_granite_lower" | "ore_diorite_upper" | "ore_diorite_lower" | "ore_andesite_upper" | "ore_andesite_lower" => {
                self.place_ore_simple(chunk, local_x, local_z, ground_y - 1, "stone_variant");
            }
            "ore_dirt" | "ore_gravel" | "ore_clay" | "ore_tuff" => {
                self.place_disk_simple(chunk, local_x, local_z, ground_y - 1, feature_path);
            }
            // Nether decorations
            "blackstone_blobs" => {
                self.place_blackstone_blobs(chunk, local_x, local_z, ground_y);
            }
            "basalt_blobs" => {
                self.place_basalt_blobs(chunk, local_x, local_z, ground_y);
            }
            "basalt_pillar" => {
                self.place_basalt_pillar(chunk, local_x, local_z, ground_y);
            }
            "small_basalt_columns" => {
                self.place_small_basalt_columns(chunk, local_x, local_z, ground_y);
            }
            "chorus_plant" => {
                self.place_chorus_plant(chunk, local_x, local_z, ground_y);
            }
            "glowstone" | "glowstone_extra" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GLOWSTONE);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            "delta" => {
                let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BASALT);
                self.place_simple_block(chunk, local_x, local_z, ground_y, block_id);
            }
            // Fossils
            "fossil_upper" | "fossil_lower" => {
                self.place_fossil(chunk, local_x, local_z, ground_y);
            }
            // End decorations
            "end_island_decorated" => {
                self.place_end_island_decorated(chunk, local_x, local_z, ground_y);
            }
            "end_spike" => {
                self.place_end_spike(chunk, local_x, local_z, ground_y);
            }
            // Void
            "void_start_platform" => {
                self.place_void_start_platform(chunk, local_x, local_z, ground_y);
            }
            _ => {}
        }
    }

    fn place_tree_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32, tree_type: &str) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let trunk_id = match tree_type {
            "oak" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LOG),
            "spruce" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPRUCE_LOG),
            "birch" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BIRCH_LOG),
            "acacia" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ACACIA_LOG),
            "jungle" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::JUNGLE_LOG),
            "dark_oak" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_LOG),
            "mangrove" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MANGROVE_LOG),
            "cherry" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CHERRY_LOG),
            "crimson" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CRIMSON_STEM),
            "warped" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::WARPED_STEM),
            _ => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LOG),
        };

        let leaf_id = match tree_type {
            "oak" | "jungle" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LEAVES),
            "spruce" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPRUCE_LEAVES),
            "birch" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BIRCH_LEAVES),
            "acacia" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ACACIA_LEAVES),
            "dark_oak" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_LEAVES),
            "cherry" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CHERRY_LEAVES),
            "mangrove" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MANGROVE_LEAVES),
            _ => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LEAVES),
        };

        let trunk_height = self.next_int(4, 8);

        for dy in 0..trunk_height {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, trunk_id);
            }
        }

        let leaf_start = ground_y + trunk_height - 2;
        let leaf_end = ground_y + trunk_height + 1;
        for dy in leaf_start..leaf_end {
            let radius = if dy == leaf_start { 1 } else { 2 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                    let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                    let ly = (dy - min_y) as usize;
                    if lx < 16 && lz < 16 && ly < height && ly >= min_y as usize {
                        let is_edge = dx.abs() == radius || dz.abs() == radius || dy == leaf_end - 1;
                        if is_edge {
                            chunk.set_relative_block(lx, ly, lz, leaf_id);
                        }
                    }
                }
            }
        }
    }

    fn place_flower_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let flower_idx = (self.random.next_i32() as usize) % 7;
        let flower_id = match flower_idx {
            0 => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::POPPY),
            1 => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DANDELION),
            2 => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OXEYE_DAISY),
            3 => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CORNFLOWER),
            4 => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::LILY_OF_THE_VALLEY),
            5 => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ALLIUM),
            _ => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BLUE_ORCHID),
        };

        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, flower_id);
    }

    fn place_grass_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let grass_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GRASS_BLOCK);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, grass_id);
    }

    fn place_tall_grass(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y + 1 >= min_y + height as i32 {
            return;
        }

        let tall_grass_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::TALL_GRASS);
        let relative_y = (ground_y - min_y) as usize;
        let relative_y_top = relative_y + 1;
        
        if relative_y < height && relative_y_top < height {
            chunk.set_relative_block(local_x, relative_y, local_z, tall_grass_id);
            chunk.set_relative_block(local_x, relative_y_top, local_z, tall_grass_id);
        }
    }

    fn place_tall_flowers(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y + 1 >= min_y + height as i32 {
            return;
        }

        let sunflower_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SUNFLOWER);
        let relative_y = (ground_y - min_y) as usize;
        let relative_y_top = relative_y + 1;
        
        if relative_y < height && relative_y_top < height {
            chunk.set_relative_block(local_x, relative_y, local_z, sunflower_id);
            chunk.set_relative_block(local_x, relative_y_top, local_z, sunflower_id);
        }
    }

    fn place_mushroom_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32, _feature: &str) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let block_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::RED_MUSHROOM_BLOCK);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, block_id);
    }

    fn place_pumpkin_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let pumpkin_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::PUMPKIN);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, pumpkin_id);
    }

    fn place_melon_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let melon_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MELON);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, melon_id);
    }

    fn place_sugar_cane_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let sugar_cane_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SUGAR_CANE);

        for dy in 0..3 {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, sugar_cane_id);
            }
        }
    }

    fn place_spring_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, y: i32, _feature: &str) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if y as usize >= height {
            return;
        }

        let fluid_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::WATER);
        chunk.set_relative_block(local_x, (y - min_y) as usize, local_z, fluid_id);
    }

    fn place_lava_lake_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        let lava_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::LAVA);

        for dy in 0..4 {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, lava_id);
            }
        }
    }

    fn place_geode_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, base_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if base_y as usize >= height || base_y < min_y {
            return;
        }

        let outer_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OBSIDIAN);
        let inner_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AMETHYST_BLOCK);

        for dy in -3..=3 {
            for dx in -3..=3 {
                for dz in -3..=3 {
                    let dist = (dx*dx + dy*dy + dz*dz) as f64;
                    if dist <= 9.0 && dist > 4.0 {
                        let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                        let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                        let ly = (base_y - min_y + dy) as usize;
                        if lx < 16 && lz < 16 && ly < height && ly >= min_y as usize {
                            let block_id = if dist <= 6.0 { inner_id } else { outer_id };
                            chunk.set_relative_block(lx, ly, lz, block_id);
                        }
                    }
                }
            }
        }
    }

    fn place_monster_room_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, room_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;
        
        if room_y as usize >= height || room_y < min_y {
            return;
        }

        let air_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        let cobble_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);

        for dx in -4..=4 {
            for dz in -4..=4 {
                for dy in -1..=4 {
                    let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                    let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                    let ly = (room_y - min_y + dy) as usize;
                    
                    if lx < 16 && lz < 16 && ly < height && ly >= min_y as usize {
                        let abs_dx = dx.abs();
                        let abs_dz = dz.abs();
                        let abs_dy = dy.abs();
                        
                        if abs_dx <= 4 && abs_dz <= 4 && abs_dy <= 4 {
                            if abs_dx == 4 || abs_dz == 4 || abs_dy == 4 {
                                chunk.set_relative_block(lx, ly, lz, air_id);
                            } else if abs_dx >= 3 && abs_dz >= 3 || abs_dx >= 3 && abs_dy >= 3 || abs_dz >= 3 && abs_dy >= 3 {
                                chunk.set_relative_block(lx, ly, lz, cobble_id);
                            }
                        }
                    }
                }
            }
        }
    }

    fn place_ore_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, y: i32, ore_type: &str) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if y as usize >= height || y < min_y {
            return;
        }

        let block_id = match ore_type {
            "coal" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COAL_ORE),
            "iron" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::IRON_ORE),
            "gold" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GOLD_ORE),
            "copper" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COPPER_ORE),
            "redstone" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::REDSTONE_ORE),
            "lapis" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::LAPIS_ORE),
            "diamond" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DIAMOND_ORE),
            "emerald" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::EMERALD_ORE),
            "ancient_debris" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ANCIENT_DEBRIS),
            "nether" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::NETHER_QUARTZ_ORE),
            "stone_variant" => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GRANITE),
            _ => REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE),
        };

        chunk.set_relative_block(local_x, (y - min_y) as usize, local_z, block_id);
    }

    fn place_disk_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, y: i32, feature: &str) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if y as usize >= height || y < min_y {
            return;
        }

        let block_id = if feature.contains("sand") {
            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SAND)
        } else if feature.contains("gravel") {
            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GRAVEL)
        } else if feature.contains("clay") {
            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CLAY)
        } else if feature.contains("grass") {
            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DIRT)
        } else {
            return;
        };

        chunk.set_relative_block(local_x, (y - min_y) as usize, local_z, block_id);
    }

    fn place_snow_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let snow_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SNOW_BLOCK);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, snow_id);
    }

    fn place_simple_block(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, y: i32, block_id: BlockStateId) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if y as usize >= height || y < min_y {
            return;
        }

        chunk.set_relative_block(local_x, (y - min_y) as usize, local_z, block_id);
    }

    fn place_waterlily_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, water_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if water_y as usize >= height || water_y < min_y {
            return;
        }

        let lily_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::LILY_PAD);
        chunk.set_relative_block(local_x, (water_y - min_y) as usize, local_z, lily_id);
    }

    fn place_sea_pickle_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, water_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if water_y as usize >= height || water_y < min_y {
            return;
        }

        let pickle_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SEA_PICKLE);
        chunk.set_relative_block(local_x, (water_y - min_y) as usize, local_z, pickle_id);
    }

    fn place_kelp(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, water_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if water_y as usize >= height || water_y < min_y {
            return;
        }

        let kelp_plant = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::KELP_PLANT);
        let kelp = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::KELP);

        let stalk_height = self.next_int(2, 24);
        for dy in 0..stalk_height {
            let y = water_y - dy;
            if y >= min_y {
                let relative_y = (y - min_y) as usize;
                if relative_y < height {
                    let block = if dy == 0 { kelp_plant } else { kelp };
                    chunk.set_relative_block(local_x, relative_y, local_z, block);
                }
            }
        }
    }

    fn place_seagrass(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, water_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if water_y as usize >= height || water_y < min_y {
            return;
        }

        let seagrass = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::SEAGRASS);
        let grass_height = self.next_int(1, 4);
        
        for dy in 0..grass_height {
            let y = water_y - dy;
            if y >= min_y {
                let relative_y = (y - min_y) as usize;
                if relative_y < height {
                    chunk.set_relative_block(local_x, relative_y, local_z, seagrass);
                }
            }
        }
    }

    fn place_warm_ocean_vegetation(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let sea_pickle = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::SEA_PICKLE);
        
        if self.next_int(0, 100) < 15 {
            let count = self.next_int(1, 4);
            for i in 0..count {
                let y = ground_y + i as i32;
                if y < min_y + height as i32 {
                    let relative_y = (y - min_y) as usize;
                    if relative_y < height {
                        chunk.set_relative_block(local_x, relative_y, local_z, sea_pickle);
                    }
                }
            }
        }
    }

    fn place_rooted_azalea(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let dirt_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ROOTED_DIRT);
        let azalea_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AZALEA);

        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, dirt_id);
        
        let above_y = ground_y + 1;
        if above_y < min_y + height as i32 {
            chunk.set_relative_block(local_x, (above_y - min_y) as usize, local_z, azalea_id);
        }
    }

    fn place_bamboo_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let bamboo_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BAMBOO);
        let height = self.next_int(1, 4);

        for dy in 0..height {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < chunk.sections().sections.len() * 16 {
                chunk.set_relative_block(local_x, ly, local_z, bamboo_id);
            }
        }
    }

    fn place_cactus_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height {
            return;
        }

        let cactus_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CACTUS);
        let cactus_height = self.next_int(1, 3);

        for dy in 0..cactus_height {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, cactus_id);
            }
        }
    }

    fn place_desert_well(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let sandstone_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SANDSTONE);
        let water_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::WATER);

        for dx in -1..=1 {
            for dz in -1..=1 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height {
                    if dx == 0 && dz == 0 {
                        chunk.set_relative_block(lx, ly, lz, water_id);
                    } else {
                        chunk.set_relative_block(lx, ly, lz, sandstone_id);
                    }
                }
            }
        }
    }

    fn place_moss_patch(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let moss_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MOSS_BLOCK);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, moss_id);
    }

    fn place_bush(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();

        if ground_y < min_y {
            return;
        }

        let fern_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::FERN);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, fern_id);
    }

    fn place_forest_rock(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let cobble_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        
        let size = self.next_int(1, 3);
        for dx in -size..=size {
            for dz in -size..=size {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height {
                    chunk.set_relative_block(lx, ly, lz, cobble_id);
                }
            }
        }
    }

    fn place_ice_spike(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let ice_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ICE);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, ice_id);
    }

    fn place_iceberg(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let ice_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::PACKED_ICE);
        
        for dx in -2..=2 {
            for dz in -2..=2 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height {
                    if dx.abs() <= 1 && dz.abs() <= 1 {
                        chunk.set_relative_block(lx, ly, lz, ice_id);
                    }
                }
            }
        }
    }

    fn place_ice_simple(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let ice_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ICE);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, ice_id);
    }

    fn place_sweet_berry_bush(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let bush_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SWEET_BERRY_BUSH);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, bush_id);
    }

    fn place_weeping_vines(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        let vine_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::WEEPING_VINES);
        
        let vine_length = self.next_int(5, 20);
        for dy in 0..vine_length {
            let y = ground_y - dy;
            if y >= min_y && y < min_y + height as i32 {
                let relative_y = (y - min_y) as usize;
                if relative_y < height {
                    chunk.set_relative_block(local_x, relative_y, local_z, vine_id);
                }
            }
        }
    }

    fn place_twisting_vines(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        let vine_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::TWISTING_VINES);
        
        let vine_length = self.next_int(10, 25);
        for dy in 0..vine_length {
            let y = ground_y + dy;
            if y >= min_y && y < min_y + height as i32 {
                let relative_y = (y - min_y) as usize;
                if relative_y < height {
                    chunk.set_relative_block(local_x, relative_y, local_z, vine_id);
                }
            }
        }
    }

    fn place_sculk_vein(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let vein_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SCULK_VEIN);
        
        for dx in -1..=1 {
            for dz in -1..=1 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height && self.next_int(0, 10) < 7 {
                    chunk.set_relative_block(lx, ly, lz, vein_id);
                }
            }
        }
    }

    fn place_sculk_patch(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let shrieker_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SCULK_SHRIEKER);
        
        for dx in -2..=2 {
            for dz in -2..=2 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height && self.next_int(0, 10) < 3 {
                    chunk.set_relative_block(lx, ly, lz, shrieker_id);
                }
            }
        }
    }

    fn place_dripstone_cluster(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, base_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if base_y as usize >= height || base_y < min_y {
            return;
        }

        let dripstone_block = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DRIPSTONE_BLOCK);
        let count = self.next_int(1, 5);

        for _ in 0..count {
            let dx = self.next_int(-2, 2);
            let dz = self.next_int(-2, 2);
            let dy = self.next_int(0, 3);
            let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
            let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
            let ly = (base_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(lx, ly, lz, dripstone_block);
            }
        }
    }

    fn place_pointed_dripstone(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, base_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if base_y as usize >= height || base_y < min_y {
            return;
        }

        if self.next_int(0, 10) < 4 {
            let pointed_dripstone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::POINTED_DRIPSTONE);
            let length = self.next_int(2, 7);
            
            for dy in 0..length {
                let ly = (base_y - min_y + dy) as usize;
                if ly < height {
                    chunk.set_relative_block(local_x, ly, local_z, pointed_dripstone);
                }
            }
        }
    }

    fn place_fossil(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let bone_id = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BONE_BLOCK);
        
        for dx in -3..=3 {
            for dz in -3..=3 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height && self.next_int(0, 10) < 4 {
                    chunk.set_relative_block(lx, ly, lz, bone_id);
                }
            }
        }
    }

    fn place_end_island_decorated(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let end_stone = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::END_STONE);
        chunk.set_relative_block(local_x, (ground_y - min_y) as usize, local_z, end_stone);
    }

    fn place_end_spike(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let obsidian = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OBSIDIAN);
        
        for dy in 0..self.next_int(5, 15) {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, obsidian);
            }
        }
    }

    fn place_void_start_platform(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let obsidian = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OBSIDIAN);
        
        for dx in -2..=2 {
            for dz in -2..=2 {
                let px = (local_x as i32 + dx).clamp(0, 15) as usize;
                let pz = (local_z as i32 + dz).clamp(0, 15) as usize;
                
                let py = ground_y;
                if py >= min_y {
                    let relative_y = (py - min_y) as usize;
                    if relative_y < height {
                        chunk.set_relative_block(px, relative_y, pz, obsidian);
                    }
                }
            }
        }
    }

    fn place_blackstone_blobs(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let blackstone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BLACKSTONE);
        
        for dx in -1..=1 {
            for dz in -1..=1 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height && self.next_int(0, 10) < 6 {
                    chunk.set_relative_block(lx, ly, lz, blackstone);
                }
            }
        }
    }

    fn place_basalt_blobs(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let basalt = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BASALT);
        
        for dx in -1..=1 {
            for dz in -1..=1 {
                let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                let ly = (ground_y - min_y) as usize;
                
                if lx < 16 && lz < 16 && ly < height && self.next_int(0, 10) < 6 {
                    chunk.set_relative_block(lx, ly, lz, basalt);
                }
            }
        }
    }

    fn place_basalt_pillar(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let basalt = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BASALT);
        let length = self.next_int(5, 20);
        
        for dy in 0..length {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, basalt);
            }
        }
    }

    fn place_small_basalt_columns(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let basalt = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BASALT);
        let length = self.next_int(2, 5);
        
        for dy in 0..length {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dz == 0 { continue; }
                        let lx = (local_x as i32 + dx).clamp(0, 15) as usize;
                        let lz = (local_z as i32 + dz).clamp(0, 15) as usize;
                        if lx < 16 && lz < 16 {
                            chunk.set_relative_block(lx, ly, lz, basalt);
                        }
                    }
                }
            }
        }
    }

    fn place_chorus_plant(&mut self, chunk: &ChunkAccess, local_x: usize, local_z: usize, ground_y: i32) {
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        if ground_y as usize >= height || ground_y < min_y {
            return;
        }

        let chorus_flower = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CHORUS_FLOWER);
        
        for dy in 0..self.next_int(3, 8) {
            let ly = (ground_y - min_y + dy) as usize;
            if ly < height {
                chunk.set_relative_block(local_x, ly, local_z, chorus_flower);
            }
        }
    }
}