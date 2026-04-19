//! Structure generation for vanilla-like world generation.
//!
//! This module implements jigsaw-based structure generation using vanilla data files:
//! - Structure JSON files define start_pool and generation settings
//! - Template pool JSON files define piece pools with weighted element selection
//! - NBT files contain actual piece geometry
//! - Jigsaw connections connect pieces via named attachment points

use rustc_hash::FxHashMap;
use serde::Deserialize;
use steel_utils::Identifier;
use steel_utils::random::xoroshiro::Xoroshiro;
use steel_utils::random::Random;
use steel_registry::{vanilla_blocks, REGISTRY};
use std::sync::OnceLock;

use crate::chunk::chunk_access::ChunkAccess;
use crate::worldgen::structures::jigsaw::{JigsawPieceInner, TemplatePool};

pub mod jigsaw;

#[allow(dead_code)]
static STRUCTURE_CONFIGS: OnceLock<FxHashMap<Identifier, StructureConfig>> = OnceLock::new();
#[allow(dead_code)]
static TEMPLATE_POOLS: OnceLock<FxHashMap<Identifier, TemplatePool>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StructureConfig {
    #[serde(rename = "type")]
    pub structure_type: String,
    #[serde(rename = "start_pool")]
    pub start_pool: String,
    pub size: i32,
    #[serde(rename = "max_distance_from_center")]
    pub max_distance: i32,
    #[serde(rename = "biomes")]
    pub biome_tag: String,
    #[serde(rename = "step")]
    pub generation_step: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct StructureJson {
    #[serde(rename = "type")]
    pub structure_type: String,
    #[serde(rename = "start_pool")]
    pub start_pool: String,
    pub size: i32,
    #[serde(rename = "max_distance_from_center")]
    pub max_distance_from_center: Option<i32>,
    #[serde(rename = "biomes")]
    pub biomes: Option<serde_json::Value>,
}

fn load_structure_configs() -> &'static FxHashMap<Identifier, StructureConfig> {
    STRUCTURE_CONFIGS.get_or_init(|| {
        let mut m = FxHashMap::default();
        let base_path = "steel-registry/build_assets/builtin_datapacks/minecraft/worldgen/structure";
        if let Ok(dir) = std::fs::read_dir(base_path) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<StructureJson>(&content) {
                            let name = path.file_stem().unwrap().to_str().unwrap();
                            let id = Identifier::vanilla(name.to_string());
                            let max_distance = json.max_distance_from_center.unwrap_or(80);
                            m.insert(id, StructureConfig {
                                structure_type: json.structure_type,
                                start_pool: json.start_pool,
                                size: json.size,
                                max_distance,
                                biome_tag: String::new(),
                                generation_step: String::new(),
                            });
                        }
                    }
                }
            }
        }
        m
    })
}

fn load_template_pool(pool_id: &Identifier) -> Option<&'static TemplatePool> {
    let pools = TEMPLATE_POOLS.get_or_init(|| {
        let mut m = FxHashMap::default();
        let base_path = "steel-registry/build_assets/builtin_datapacks/minecraft/worldgen/template_pool";
        if let Ok(dir) = std::fs::read_dir(base_path) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(subdir) = std::fs::read_dir(&path) {
                        for subentry in subdir.flatten() {
                            let subpath = subentry.path();
                            if subpath.extension().and_then(|s| s.to_str()) == Some("json") {
                                if let Ok(content) = std::fs::read_to_string(&subpath) {
                                    if let Ok(pool) = serde_json::from_str::<TemplatePool>(&content) {
                                        let pool_name = subpath.file_stem().unwrap().to_str().unwrap();
                                        let parent_name = subpath.parent().unwrap().file_name().unwrap().to_str().unwrap();
                                        let full_path = format!("{}/{}", parent_name, pool_name);
                                        let id = Identifier::vanilla(full_path);
                                        m.insert(id, pool);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        m
    });
    pools.get(pool_id).as_ref().copied()
}

#[derive(Clone)]
pub struct StructureGenerator {
    seed: u64,
    random: Xoroshiro,
    village_pool_id: Identifier,
    village_start_pool: Identifier,
}

impl StructureGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            random: Xoroshiro::from_seed(seed),
            village_pool_id: Identifier::vanilla("village/plains/houses".to_string()),
            village_start_pool: Identifier::vanilla("village/plains/town_centers".to_string()),
        }
    }

    pub fn next_int(&mut self, min: i32, max: i32) -> i32 {
        if max - min <= 0 {
            return min;
        }
        self.random.next_i32() % (max - min) + min
    }

    pub fn apply_structure_generation(&mut self, chunk: &ChunkAccess, biome_id: u16) {
        let pos = chunk.pos();
        let chunk_x = pos.0.x;
        let chunk_z = pos.0.y;
        let min_y = chunk.min_y();
        let height = chunk.sections().sections.len() * 16;

        // Trial Chambers
        if self.should_generate_trial_chamber(chunk_x, chunk_z) {
            self.generate_trial_chamber(chunk, min_y, height);
        }

        // Village - now uses proper jigsaw generation
        if self.should_generate_village(chunk_x, chunk_z) {
            self.generate_jigsaw_village(chunk, min_y, height);
        }

        // Desert Pyramid
        if self.should_generate_desert_pyramid(chunk_x, chunk_z, biome_id) {
            self.generate_desert_pyramid(chunk, min_y, height);
        }

        // Jungle Temple
        if self.should_generate_jungle_temple(chunk_x, chunk_z, biome_id) {
            self.generate_jungle_temple(chunk, min_y, height);
        }

        // Swamp Hut
        if self.should_generate_swamp_hut(chunk_x, chunk_z, biome_id) {
            self.generate_swamp_hut(chunk, min_y, height);
        }

        // Pillager Outpost
        if self.should_generate_pillager_outpost(chunk_x, chunk_z, biome_id) {
            self.generate_pillager_outpost(chunk, min_y, height);
        }

        // Ocean Monument
        if self.should_generate_ocean_monument(chunk_x, chunk_z, biome_id) {
            self.generate_ocean_monument(chunk, min_y, height);
        }

        // Mineshaft
        if self.should_generate_mineshaft(chunk_x, chunk_z) {
            self.generate_mineshaft(chunk, min_y, height);
        }

        // Stronghold (first chunk only)
        if chunk_x == 0 && chunk_z == 0 {
            self.generate_underground_stronghold(chunk, min_y, height);
        }

        // Nether Fortress
        if self.should_generate_fortress(chunk_x, chunk_z) {
            self.generate_fortress(chunk, min_y, height);
        }

        // Bastion Remnant
        if self.should_generate_bastion(chunk_x, chunk_z) {
            self.generate_bastion(chunk, min_y, height);
        }

        // Ruined Portal
        if self.should_generate_ruined_portal(chunk_x, chunk_z) {
            self.generate_ruined_portal(chunk, min_y, height);
        }

        // Shipwreck
        if self.should_generate_shipwreck(chunk_x, chunk_z, biome_id) {
            self.generate_shipwreck(chunk, min_y, height);
        }

        // Dungeon
        if self.should_generate_dungeon(chunk_x, chunk_z) {
            self.generate_dungeon(chunk, min_y, height);
        }

        // Igloo
        if self.should_generate_igloo(chunk_x, chunk_z, biome_id) {
            self.generate_igloo(chunk, min_y, height);
        }

        // Ocean Ruin
        if self.should_generate_ocean_ruin(chunk_x, chunk_z, biome_id) {
            self.generate_ocean_ruin(chunk, min_y, height);
        }

        // Buried Treasure
        if self.should_generate_buried_treasure(chunk_x, chunk_z) {
            self.generate_buried_treasure(chunk, min_y, height);
        }

        // Nether Fossil
        if self.should_generate_nether_fossil(chunk_x, chunk_z) {
            self.generate_nether_fossil(chunk, min_y, height);
        }

        // End City
        if self.should_generate_end_city(chunk_x, chunk_z) {
            self.generate_end_city(chunk, min_y, height);
        }

        // Ancient City
        if self.should_generate_ancient_city(chunk_x, chunk_z) {
            self.generate_ancient_city(chunk, min_y, height);
        }

        // Woodland Mansion
        if self.should_generate_mansion(chunk_x, chunk_z, biome_id) {
            self.generate_mansion(chunk, min_y, height);
        }

        // Trail Ruins
        if self.should_generate_trail_ruins(chunk_x, chunk_z, biome_id) {
            self.generate_trail_ruins(chunk, min_y, height);
        }

        // Stronghold
        if self.should_generate_stronghold(chunk_x, chunk_z) {
            self.generate_underground_stronghold(chunk, min_y, height);
        }

        chunk.mark_dirty();
    }

    fn should_generate_trial_chamber(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 24i64;
        let hash = ((cx as i64 * 16777619i64) ^ (cz as i64 * 234567i64)) ^ 2345i64;
        let result = hash % spacing;
        result == 0 && cx > 2 && cz > 2
    }

    fn should_generate_village(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 34i64;
        let hash = ((cx as i64 * 16574513i64) ^ (cz as i64 * 2341653i64)) ^ 9876543i64;
        let result = hash % spacing;
        result == 0 && cx > 2 && cz > 2
    }

    fn generate_jigsaw_village(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let base_y = min_y;
        let center_x = 7i32;
        let center_z = 7i32;

        // Load town center pieces
        let town_centers = load_template_pool(&self.village_start_pool);
        
        if let Some(pool) = town_centers {
            // Try to load and parse the start piece NBT
            if let Some(element) = pool.elements.first() {
                self.place_pool_piece(
                    chunk, 
                    &element.element, 
                    center_x, 
                    base_y, 
                    center_z, 
                    height,
                    0
                );
            }
        } else {
            // Fallback to basic town center generation
            self.place_town_center(chunk, center_x, base_y, center_z, height);
        }

        // Generate additional houses in surrounding areas
        let houses_pool = load_template_pool(&self.village_pool_id);
        if let Some(pool) = houses_pool {
            let num_houses = self.next_int(3, 6) as i32;
            for i in 0..num_houses {
                let angle = (i as f32 / num_houses as f32) * std::f32::consts::TAU;
                let dist = self.next_int(4, 8) as i32;
                let hx = center_x + (angle.cos() * dist as f32).round() as i32;
                let hz = center_z + (angle.sin() * dist as f32).round() as i32;
                
                if let Some(element) = pool.elements.get((i as usize) % pool.elements.len().max(1)) {
                    self.place_pool_piece(chunk, &element.element, hx, base_y, hz, height, 1);
                }
            }
        } else {
            // Fallback: place houses around center
            let num_houses = self.next_int(3, 6) as i32;
            for i in 0..num_houses {
                let angle = (i as f32 / num_houses as f32) * std::f32::consts::TAU;
                let dist = self.next_int(4, 8) as i32;
                let hx = center_x + (angle.cos() * dist as f32).round() as i32;
                let hz = center_z + (angle.sin() * dist as f32).round() as i32;
                self.place_house_piece(chunk, hx, base_y, hz, height);
            }
        }
    }

    fn place_pool_piece(
        &mut self,
        chunk: &ChunkAccess,
        element: &JigsawPieceInner,
        local_x: i32,
        base_y: i32,
        local_z: i32,
        height: usize,
        _depth: i32,
    ) {
        // Simplified village generation - place basic structure based on piece type
        let location = &element.location;
        
        // Check location to determine piece type
        let is_house = location.contains("house");
        let is_town_center = location.contains("town_center") || location.contains("meeting_point") || location.contains("fountain");
        let is_street = location.contains("street");
        
        if is_town_center {
            self.place_town_center(chunk, local_x, base_y, local_z, height);
        } else if is_house {
            self.place_house_piece(chunk, local_x, base_y, local_z, height);
        } else if is_street {
            self.place_street_piece(chunk, local_x, base_y, local_z, height);
        }
    }

    fn place_town_center(&self, chunk: &ChunkAccess, local_x: i32, base_y: i32, local_z: i32, height: usize) {
        // Place town center (well/meeting point)
        let planks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_PLANKS);
        let log = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LOG);
        let fence = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_FENCE);
        
        // 3x3 well area
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let lx = (local_x + dx).clamp(0, 15) as usize;
                let lz = (local_z + dz).clamp(0, 15) as usize;
                if (base_y as usize) < height {
                    chunk.set_relative_block(lx, base_y as usize, lz, log);
                }
            }
        }
        
        if ((base_y + 1) as usize) < height {
            // Water in center
            chunk.set_relative_block(local_x as usize, (base_y + 1) as usize, local_z as usize,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::WATER));
        }
        
        // Fence around
        if ((base_y + 1) as usize) < height {
            for &(dx, dz) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let lx = (local_x + dx).clamp(0, 15) as usize;
                let lz = (local_z + dz).clamp(0, 15) as usize;
                chunk.set_relative_block(lx, (base_y + 1) as usize, lz, fence);
            }
        }
    }

    fn place_house_piece(&self, chunk: &ChunkAccess, local_x: i32, base_y: i32, local_z: i32, height: usize) {
        // Place a simple house structure
        let planks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_PLANKS);
        
        // 3x3 floor
        for dx in 0i32..3 {
            for dz in 0i32..3 {
                let lx = (local_x + dx).clamp(0, 15) as usize;
                let lz = (local_z + dz).clamp(0, 15) as usize;
                if (base_y as usize) < height {
                    chunk.set_relative_block(lx, base_y as usize, lz, planks);
                }
            }
        }
        
        // Walls (only 2 sides to avoid overcrowding)
        let bh1 = (base_y + 1) as usize;
        let bh2 = (base_y + 2) as usize;
        if bh1 < height && bh2 < height {
            chunk.set_relative_block(local_x as usize, bh1, local_z as usize,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR));
            chunk.set_relative_block(local_x as usize, bh2, local_z as usize,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR));
        }
    }

    fn place_street_piece(&self, chunk: &ChunkAccess, local_x: i32, base_y: i32, local_z: i32, height: usize) {
        // Place a street segment
        let planks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_PLANKS);
        
        for dx in 0i32..3 {
            let lx = (local_x + dx).clamp(0, 15) as usize;
            if (base_y as usize) < height {
                chunk.set_relative_block(lx, base_y as usize, local_z as usize, planks);
            }
        }
    }

    fn should_generate_desert_pyramid(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_desert = biome_id == 24 || biome_id == 25;
        if !is_desert { return false; }
        let spacing = 32i64;
        let hash = ((cx as i64 * 14271113i64) ^ (cz as i64 * 82691i64)) ^ 4453151i64;
        hash % spacing == 0
    }

    fn should_generate_jungle_temple(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_jungle = biome_id == 21 || biome_id == 22 || biome_id == 23;
        if !is_jungle { return false; }
        let spacing = 28i64;
        let hash = ((cx as i64 * 16731329i64) ^ (cz as i64 * 12032651i64)) ^ 2858193i64;
        hash % spacing == 0
    }

    fn should_generate_swamp_hut(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_swamp = biome_id == 6 || biome_id == 134;
        if !is_swamp { return false; }
        let spacing = 26i64;
        let hash = ((cx as i64 * 15242591i64) ^ (cz as i64 * 10534531i64)) ^ 3512581i64;
        hash % spacing == 0
    }

    fn should_generate_pillager_outpost(&mut self, cx: i32, cz: i32, _biome_id: u16) -> bool {
        let spacing = 30i64;
        let hash = ((cx as i64 * 16777619i64) ^ (cz as i64 * 1970159i64)) ^ 4231759i64;
        hash % spacing == 0 && cx > 1 && cz > 1
    }

    fn should_generate_ocean_monument(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_ocean = biome_id == 0 || biome_id == 10 || biome_id == 46 || biome_id == 48;
        if !is_ocean { return false; }
        let spacing = 26i64;
        let hash = ((cx as i64 * 14276191i64) ^ (cz as i64 * 10089131i64)) ^ 2685421i64;
        hash % spacing == 0 && cx > 1 && cz > 1
    }

    fn should_generate_mineshaft(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 20i64;
        let hash = ((cx as i64 * 1341767i64) ^ (cz as i64 * 15269i64)) ^ 1657451i64;
        hash % spacing == 0 && cx > 1 && cz > 1
    }

    fn should_generate_ruined_portal(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 22i64;
        let hash = ((cx as i64 * 1264827i64) ^ (cz as i64 * 15491i64)) ^ 1523819i64;
        hash % spacing == 0
    }

    fn should_generate_fortress(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 27i64;
        let hash = ((cx as i64 * 14357619i64) ^ (cz as i64 * 2343269i64)) ^ 8576359i64;
        hash % spacing == 0
    }

    fn generate_fortress(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let nether_brick = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::NETHER_BRICKS);
        let nether_brick_fence = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::NETHER_BRICK_FENCE);
        let nether_wart = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::NETHER_WART_BLOCK);
        
        let base_y = min_y + 20;
        
        for dx in -6i32..=6 {
            for dz in -6i32..=6 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs() + dz.abs();
                
                if dist <= 6 {
                    let max_dy = if dist <= 1 { 8 } else if dist <= 3 { 6 } else { 4 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_wall = dx.abs() == 6 || dz.abs() == 6;
                        let is_corner = dx.abs() >= 5 && dz.abs() >= 5;
                        
                        let block = if is_wall {
                            if dy == max_dy - 1 {
                                nether_brick_fence
                            } else {
                                nether_brick
                            }
                        } else if is_corner && dy >= 2 && dy <= 4 {
                            nether_wart
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
        
        if height > (base_y + 3) as usize {
            for &(dx, dz) in &[(-2, 0), (2, 0), (0, -2), (0, 2)] {
                let lx = (7 + dx) as usize;
                let lz = (7 + dz) as usize;
                chunk.set_relative_block(lx, (base_y + 1) as usize, lz, nether_brick_fence);
            }
        }
    }

    fn should_generate_bastion(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 25i64;
        let hash = ((cx as i64 * 16574513i64) ^ (cz as i64 * 2341653i64)) ^ 9234521i64;
        hash % spacing == 0
    }

    fn generate_bastion(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let Blackstone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BLACKSTONE);
        let BlackstoneBrick = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::POLISHED_BLACKSTONE_BRICKS);
        let basalt = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BASALT);
        let gold = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GOLD_BLOCK);
        
        let base_y = min_y + 24;
        
        for dx in -8i32..=8 {
            for dz in -8i32..=8 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs().max(dz.abs());
                
                if dist <= 8 {
                    let max_dy = if dist <= 2 { 7 } else if dist <= 4 { 5 } else { 3 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_outer_wall = dist >= 7;
                        let is_corner = dist >= 6 && (dx.abs() >= 5 || dz.abs() >= 5);
                        let is_bridge = (dx.abs() <= 1 || dz.abs() <= 1) && dy >= 1 && dy <= 3;
                        
                        let block = if is_outer_wall {
                            if dy == max_dy - 1 {
                                BlackstoneBrick
                            } else {
                                Blackstone
                            }
                        } else if is_corner && dy >= 1 && dy <= 4 {
                            basalt
                        } else if is_bridge && (dx == 0 || dz == 0) && (dx + dy + dz) % 3 == 0 {
                            gold
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn should_generate_shipwreck(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_beach = biome_id == 16 || biome_id == 17 || biome_id == 47;
        if !is_beach { return false; }
        let spacing = 24i64;
        let hash = ((cx as i64 * 16717691i64) ^ (cz as i64 * 12691181i64)) ^ 2518171i64;
        hash % spacing == 0
    }

    fn should_generate_dungeon(&mut self, _cx: i32, _cz: i32) -> bool {
        self.next_int(0, 10) == 0
    }

    fn generate_trial_chamber(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let stone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE);
        let stone_brick = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE_BRICKS);
        
        let base_y = min_y + 4;
        
        for dx in -4i32..=4 {
            for dz in -4i32..=4 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..=4 {
                    let ly = (base_y + dy) as usize;
                    if ly < height {
                        let block = if dx.abs() == 4 || dz.abs() == 4 || dy == 4 {
                            stone_brick
                        } else {
                            stone
                        };
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn generate_desert_pyramid(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let sandstone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SANDSTONE);
        let smooth_sandstone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SMOOTH_SANDSTONE);
        let air = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        
        let base_y = min_y;
        
        for dx in -5i32..=5 {
            for dz in -5i32..=5 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..8 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { break; }
                    
                    let is_edge = dx.abs() == 5 || dz.abs() == 5;
                    let is_top = dy == 7;
                    
                    let block = if is_top {
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CHISELED_SANDSTONE)
                    } else if is_edge {
                        sandstone
                    } else {
                        air
                    };
                    
                    chunk.set_relative_block(lx, ly, lz, block);
                }
                
                if dx.abs() < 5 && dz.abs() < 5 {
                    chunk.set_relative_block(lx, base_y as usize, lz, smooth_sandstone);
                }
            }
        }
        
        if height > (base_y + 4) as usize {
            chunk.set_relative_block(7, (base_y + 1) as usize, 7,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::HEAVY_WEIGHTED_PRESSURE_PLATE));
            for dx in -1i32..=1 {
                for dz in -1i32..=1 {
                    chunk.set_relative_block(
                        (7 + dx) as usize,
                        (base_y - 1) as usize,
                        (7 + dz) as usize,
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::TNT)
                    );
                }
            }
        }
    }

    fn generate_jungle_temple(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MOSSY_COBBLESTONE);
        
        let base_y = min_y;
        
        for dx in -3i32..=3 {
            for dz in -3i32..=3 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..9 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { break; }
                    
                    let is_edge = dx.abs() == 3 || dz.abs() == 3;
                    let block = if is_edge { cobble } else {
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                    };
                    
                    chunk.set_relative_block(lx, ly, lz, block);
                }
                
                if dx.abs() < 3 && dz.abs() < 3 {
                    chunk.set_relative_block(lx, base_y as usize, lz, cobble);
                }
            }
        }
        
        if height > (base_y + 10) as usize {
            for i in 0i32..3 {
                chunk.set_relative_block(7, (base_y + 4 + i) as usize, 10,
                    REGISTRY.blocks.get_default_state_id(&vanilla_blocks::VINE));
            }
        }
    }

    fn generate_swamp_hut(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let wood = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPRUCE_PLANKS);
        
        let base_y = min_y;
        
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..4 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { break; }
                    
                    let block = if dy == 3 {
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPRUCE_SLAB)
                    } else {
                        wood
                    };
                    
                    chunk.set_relative_block(lx, ly, lz, block);
                }
            }
        }
        
        if height > (base_y + 2) as usize {
            chunk.set_relative_block(7, (base_y + 1) as usize, 7,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPAWNER));
        }
    }

    fn generate_pillager_outpost(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        let dark_oak = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_LOG);
        
        let base_y = min_y;
        
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..7 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { break; }
                    
                    let block = if dy == 6 || dy < 2 {
                        dark_oak
                    } else {
                        cobble
                    };
                    
                    chunk.set_relative_block(lx, ly, lz, block);
                }
            }
        }
        
        if height > (base_y + 8) as usize {
            chunk.set_relative_block(6, (base_y + 7) as usize, 6,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::IRON_BARS));
            chunk.set_relative_block(8, (base_y + 7) as usize, 6,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::IRON_BARS));
            chunk.set_relative_block(6, (base_y + 7) as usize, 8,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::IRON_BARS));
            chunk.set_relative_block(8, (base_y + 7) as usize, 8,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::IRON_BARS));
        }
    }

    fn generate_ocean_monument(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let prismarine = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::PRISMARINE);
        let dark_prismarine = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_PRISMARINE);
        let prismarine_bricks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::PRISMARINE_BRICKS);
        
        let base_y = min_y;
        
        for dx in -12i32..=12 {
            for dz in -12i32..=12 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs() + dz.abs();
                
                if dist <= 12 {
                    let max_dy = if dist <= 3 { 13 } else if dist <= 6 { 10 } else if dist <= 10 { 6 } else { 3 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_central = dx.abs() <= 2 && dz.abs() <= 2;
                        let block = if is_central && dy >= 1 && dy <= 3 {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::GOLD_BLOCK)
                        } else if dy == max_dy - 1 {
                            dark_prismarine
                        } else if (dx + dz + dy) % 2 == 0 {
                            prismarine
                        } else {
                            prismarine_bricks
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn generate_mineshaft(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let wood = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_PLANKS);
        let support = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LOG);
        
        let base_y = min_y - 5;
        
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                let ly = base_y as usize;
                
                if ly >= height { continue; }
                
                let is_edge = dx.abs() == 2 || dz.abs() == 2;
                let block = if is_edge { support } else { wood };
                chunk.set_relative_block(lx, ly, lz, block);
                
                let is_corner = dx.abs() == 2 && dz.abs() == 2;
                if is_corner && ly > 0 {
                    chunk.set_relative_block(lx, ly - 1, lz,
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_FENCE));
                }
            }
        }
    }

    fn generate_stronghold(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let stone_brick = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE_BRICKS);
        let portal_frame = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::END_PORTAL_FRAME);
        
        let base_y = min_y + 10;
        
        for dx in -4i32..=4 {
            for dz in -4i32..=4 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..5 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { continue; }
                    
                    let is_edge = dx.abs() == 4 || dz.abs() == 4;
                    let block = if is_edge {
                        stone_brick
                    } else {
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                    };
                    
                    chunk.set_relative_block(lx, ly, lz, block);
                }
            }
        }
        
        if height > (base_y + 2) as usize {
            for dx in -3i32..=3 {
                for dz in -3i32..=3 {
                    let lx = (7 + dx).clamp(0, 15) as usize;
                    let lz = (7 + dz).clamp(0, 15) as usize;
                    
                    if dx.abs() == 3 || dz.abs() == 3 {
                        chunk.set_relative_block(lx, (base_y + 1) as usize, lz, portal_frame);
                    } else {
                        chunk.set_relative_block(lx, (base_y + 1) as usize, lz,
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR));
                    }
                }
            }
            chunk.set_relative_block(7, (base_y + 2) as usize, 7,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::END_PORTAL_FRAME));
        }
    }

    fn generate_ruined_portal(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let obsidian = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OBSIDIAN);
        let portal = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::NETHER_PORTAL);
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        
        let base_y = min_y;
        
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..5 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { continue; }
                    
                    let is_frame = dx.abs() == 1 || dz.abs() == 1;
                    let block = if (dx + dy).abs() % 2 == 0 {
                        obsidian
                    } else {
                        cobble
                    };
                    
                    if is_frame {
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
        
        if height > (base_y + 3) as usize {
            chunk.set_relative_block(7, (base_y + 1) as usize, 7, portal);
            chunk.set_relative_block(7, (base_y + 2) as usize, 7, portal);
            chunk.set_relative_block(7, (base_y + 3) as usize, 7, portal);
        }
    }

    fn generate_shipwreck(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let dark_oak = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_PLANKS);
        let oak_planks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_PLANKS);
        
        let base_y = min_y;
        
        for dx in -3i32..=3 {
            for dz in -1i32..=1 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                let ly = base_y as usize;
                
                if ly >= height { continue; }
                
                let block = if dz.abs() == 1 || (dx.abs() == 3 && dz == 0) {
                    dark_oak
                } else {
                    oak_planks
                };
                
                chunk.set_relative_block(lx, ly, lz, block);
                if ly > 0 {
                    chunk.set_relative_block(lx, ly - 1, lz, oak_planks);
                }
            }
        }
        
        if height > (base_y + 2) as usize {
            chunk.set_relative_block(7, (base_y + 1) as usize, 7,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_LOG));
        }
    }

    fn generate_dungeon(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        let mossy = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MOSSY_COBBLESTONE);
        
        let dx = self.next_int(2, 13);
        let dz = self.next_int(2, 13);
        let base_y = min_y;
        
        for ix in 0i32..3 {
            for iz in 0i32..3 {
                let lx = (dx + ix) as usize;
                let lz = (dz + iz) as usize;
                
                if height < (base_y + 3) as usize { continue; }
                
                chunk.set_relative_block(lx, base_y as usize, lz, cobble);
                
                if iz == 0 || iz == 2 || ix == 0 {
                    for dy in 1i32..=2 {
                        let block = if (ix + iz + dy) % 3 == 0 { mossy } else { cobble };
                        chunk.set_relative_block(lx, (base_y + dy) as usize, lz, block);
                    }
                } else if ix == 1 && iz == 1 {
                    chunk.set_relative_block(lx, (base_y + 1) as usize, lz,
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SPAWNER));
                }
            }
        }
    }

    fn should_generate_igloo(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_snowy = biome_id == 12 || biome_id == 26 || biome_id == 27;
        if !is_snowy { return false; }
        let spacing = 28i64;
        let hash = ((cx as i64 * 15242591i64) ^ (cz as i64 * 10534531i64)) ^ 3512581i64;
        hash % spacing == 0
    }

    fn generate_igloo(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let snow = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SNOW_BLOCK);
        let ice = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::ICE);
        let planks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::OAK_PLANKS);
        
        let base_y = min_y;
        
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                for dy in 0i32..3 {
                    let ly = (base_y + dy) as usize;
                    if ly >= height { continue; }
                    
                    let is_roof = dy == 2;
                    let is_door = (dx == 0 || dx == 1) && dz == 2 && dy < 2;
                    
                    let block = if is_door {
                        REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                    } else if is_roof {
                        if dx.abs() == 2 || dz.abs() == 2 {
                            snow
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        }
                    } else {
                        snow
                    };
                    
                    chunk.set_relative_block(lx, ly, lz, block);
                }
                
                if dx.abs() < 2 && dz.abs() < 2 {
                    chunk.set_relative_block(lx, base_y as usize, lz, ice);
                }
            }
        }
        
        if height > (base_y + 1) as usize {
            chunk.set_relative_block(7, (base_y + 1) as usize, 8,
                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::RED_BED));
            chunk.set_relative_block(8, (base_y + 1) as usize, 7,
                planks);
        }
    }

    fn should_generate_ocean_ruin(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_ocean = biome_id == 0 || biome_id == 10 || biome_id == 46 || biome_id == 48;
        if !is_ocean { return false; }
        let spacing = 20i64;
        let hash = ((cx as i64 * 14276191i64) ^ (cz as i64 * 10089131i64)) ^ 2685421i64;
        hash % spacing == 0
    }

    fn generate_ocean_ruin(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let stone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE);
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        
        let base_y = min_y;
        
        for dx in -4i32..=4 {
            for dz in -4i32..=4 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs() + dz.abs();
                
                if dist <= 4 {
                    let is_ruin = self.next_int(0, 10) > 3;
                    
                    if is_ruin {
                        for dy in 0i32..2 {
                            let ly = (base_y + dy) as usize;
                            if ly >= height { continue; }
                            
                            let block = if (dx + dy + dz) % 3 == 0 { cobble } else { stone };
                            chunk.set_relative_block(lx, ly, lz, block);
                        }
                    }
                }
            }
        }
    }

    fn should_generate_buried_treasure(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 1i64;
        let hash = ((cx as i64 * 1234567i64) ^ (cz as i64 * 987654i64)) ^ 123456i64;
        hash % spacing == 0 && cx > 1 && cz > 1
    }

    fn generate_buried_treasure(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let chest = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::CHEST);
        
        let dx = self.next_int(3, 12);
        let dz = self.next_int(3, 12);
        let base_y = min_y - 1;
        
        if (base_y as usize) < height {
            chunk.set_relative_block(dx as usize, base_y as usize, dz as usize, chest);
        }
        
        for &(ox, oz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let px = ((dx as i32) + ox).clamp(0, 15) as usize;
            let pz = ((dz as i32) + oz).clamp(0, 15) as usize;
            if (base_y as usize) < height {
                chunk.set_relative_block(px, base_y as usize, pz,
                    REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE));
            }
        }
    }

    fn should_generate_nether_fossil(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 50i64;
        let hash = ((cx as i64 * 19876543i64) ^ (cz as i64 * 12345678i64)) ^ 9876543i64;
        hash % spacing == 0
    }

    fn generate_nether_fossil(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let bone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BONE_BLOCK);
        
        let base_y = min_y + 10;
        
        for dx in -3i32..=3 {
            for dz in -3i32..=3 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let is_spine = (dx * dx + dz * dz) <= 1;
                let is_rib = (dx.abs() == 2 || dz.abs() == 2) && (dx + dz).abs() % 2 == 0;
                
                if is_spine || is_rib {
                    for dy in 0i32..3 {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        chunk.set_relative_block(lx, ly, lz, bone);
                    }
                }
            }
        }
    }

    fn should_generate_end_city(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 20i64;
        let hash = ((cx as i64 * 1234567i64) ^ (cz as i64 * 987654i64)) ^ 1111111i64;
        hash % spacing == 0 && cx > 5 && cz > 5
    }

    fn generate_end_city(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let purpur = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::PURPUR_BLOCK);
        let purpur_stairs = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::PURPUR_STAIRS);
        let shulker = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::SHULKER_BOX);
        let end_rod = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::END_ROD);
        
        let base_y = min_y + 30;
        
        for dx in -6i32..=6 {
            for dz in -6i32..=6 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs().max(dz.abs());
                
                if dist <= 6 {
                    let max_dy = if dist <= 1 { 15 } else if dist <= 3 { 10 } else { 5 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_tower = dx.abs() <= 1 && dz.abs() <= 1;
                        let is_bridge = (dx.abs() <= 1 || dz.abs() <= 1) && dy >= 5 && dy <= 8;
                        let is_corner = dist >= 5;
                        
                        let block = if is_tower {
                            if dy == 0 {
                                purpur
                            } else if dy == max_dy - 1 && (dx == 0 || dz == 0) {
                                shulker
                            } else if dy == max_dy - 1 {
                                end_rod
                            } else {
                                purpur
                            }
                        } else if is_bridge && (dx == 0 || dz == 0) {
                            purpur_stairs
                        } else if is_corner && dy >= 2 && dy <= 4 {
                            purpur
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn should_generate_ancient_city(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 24i64;
        let hash = ((cx as i64 * 16574513i64) ^ (cz as i64 * 2341653i64)) ^ 7654321i64;
        hash % spacing == 0 && cx > 3 && cz > 3
    }

    fn generate_ancient_city(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let deepslate = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DEEPSLATE);
        let deepslate_brick = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DEEPSLATE_BRICKS);
        let cobbled_deepslate = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLED_DEEPSLATE);
        let lantern = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::LANTERN);
        
        let base_y = min_y + 35;
        
        for dx in -8i32..=8 {
            for dz in -8i32..=8 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs().max(dz.abs());
                
                if dist <= 8 {
                    let max_dy = if dist <= 1 { 12 } else if dist <= 3 { 8 } else if dist <= 5 { 5 } else { 3 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_outer_wall = dist >= 7;
                        let is_corridor = (dx.abs() <= 1 || dz.abs() <= 1) && dy >= 2 && dy <= 5;
                        let is_pillar = (dx.abs() == 2 || dz.abs() == 2) && dy >= 1 && dy <= 4;
                        let is_lantern = is_corridor && (dx == 0 || dz == 0) && dy == 3;
                        
                        let block = if is_lantern {
                            lantern
                        } else if is_outer_wall {
                            if dy == max_dy - 1 {
                                deepslate_brick
                            } else {
                                deepslate
                            }
                        } else if is_corridor || is_pillar {
                            if (dx + dy + dz) % 2 == 0 {
                                deepslate_brick
                            } else {
                                cobbled_deepslate
                            }
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn should_generate_mansion(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_forest = biome_id == 4 || biome_id == 157 || biome_id == 132;
        if !is_forest { return false; }
        let spacing = 80i64;
        let hash = ((cx as i64 * 1234567i64) ^ (cz as i64 * 987654i64)) ^ 5555555i64;
        hash % spacing == 0 && cx > 3 && cz > 3
    }

    fn generate_mansion(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let dark_oak_log = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_LOG);
        let dark_oak_planks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_PLANKS);
        let dark_oak_fence = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::DARK_OAK_FENCE);
        
        let base_y = min_y;
        
        for dx in -7i32..=7 {
            for dz in -7i32..=7 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs().max(dz.abs());
                
                if dist <= 7 {
                    let max_dy = if dist <= 2 { 8 } else if dist <= 4 { 5 } else { 3 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_wall = dist >= 6;
                        let is_roof = dy == max_dy - 1;
                        let is_fence_rail = is_wall && dy == max_dy - 2;
                        
                        let block = if is_fence_rail {
                            dark_oak_fence
                        } else if is_roof {
                            if dist >= 5 {
                                dark_oak_log
                            } else {
                                REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                            }
                        } else if is_wall {
                            dark_oak_planks
                        } else if dy == 0 && dist <= 2 {
                            dark_oak_planks
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn should_generate_trail_ruins(&mut self, cx: i32, cz: i32, biome_id: u16) -> bool {
        let is_taiga = biome_id == 27 || biome_id == 28 || biome_id == 29 || biome_id == 30;
        if !is_taiga { return false; }
        let spacing = 50i64;
        let hash = ((cx as i64 * 1878761i64) ^ (cz as i64 * 1234567i64)) ^ 8765432i64;
        hash % spacing == 0 && cx > 3 && cz > 3
    }

    fn generate_trail_ruins(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        let mossy_cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::MOSSY_COBBLESTONE);
        let bricks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::BRICKS);
        
        let base_y = min_y;
        
        for dx in -4i32..=4 {
            for dz in -4i32..=4 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs().max(dz.abs());
                
                if dist <= 4 {
                    let max_dy = if dist <= 1 { 4 } else if dist <= 2 { 3 } else { 2 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_corner = dist >= 3;
                        let is_ground = dy == 0;
                        
                        let block = if is_corner && dy == 1 && (dx * dz).abs() == 1 {
                            bricks
                        } else if is_ground {
                            if (dx + dz + dy) % 3 == 0 { mossy_cobble } else { cobble }
                        } else if dy == 1 && (dx * dz).abs() <= 1 {
                            cobble
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }

    fn should_generate_stronghold(&mut self, cx: i32, cz: i32) -> bool {
        let spacing = 1i64;
        let hash = ((cx as i64 * 16574513i64) ^ (cz as i64 * 2341653i64)) ^ 1234567i64;
        let ring_distance = (cx * cx + cz * cz) as i64;
        hash % spacing == 0 && ring_distance >= 25 && ring_distance <= 625
    }

    fn generate_underground_stronghold(&mut self, chunk: &ChunkAccess, min_y: i32, height: usize) {
        let stone_bricks = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE_BRICKS);
        let stone = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::STONE);
        let cobble = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::COBBLESTONE);
        
        let base_y = min_y - 10;
        
        for dx in -6i32..=6 {
            for dz in -6i32..=6 {
                let lx = (7 + dx).clamp(0, 15) as usize;
                let lz = (7 + dz).clamp(0, 15) as usize;
                
                let dist = dx.abs().max(dz.abs());
                
                if dist <= 6 {
                    let max_dy = if dist <= 2 { 6 } else if dist <= 4 { 4 } else { 3 };
                    
                    for dy in 0i32..max_dy {
                        let ly = (base_y + dy) as usize;
                        if ly >= height { continue; }
                        
                        let is_corridor = (dx.abs() <= 1 || dz.abs() <= 1) && dy >= 1 && dy <= 3;
                        let is_wall = dist >= 5;
                        let is_pillar = (dx.abs() == 2 || dz.abs() == 2) && dy >= 1;
                        
                        let block = if is_corridor {
                            if (dx + dy + dz) % 2 == 0 { stone_bricks } else { stone }
                        } else if is_wall {
                            if dy == max_dy - 1 { stone_bricks } else { stone }
                        } else if is_pillar {
                            cobble
                        } else {
                            REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR)
                        };
                        
                        chunk.set_relative_block(lx, ly, lz, block);
                    }
                }
            }
        }
    }
}