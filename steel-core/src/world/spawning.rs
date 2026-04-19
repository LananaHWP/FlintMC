//! Mob spawning system for vanilla biome-based spawning.
//!
//! Handles spawning of passive mobs (creatures), hostile mobs (monsters), ambient mobs (bats),
//! and water mobs based on biome spawn rules.

use std::sync::Arc;

use glam::DVec3;
use rand::RngExt;
use rustc_hash::FxHashMap;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::entity_types::EntityTypeRef;
use steel_registry::vanilla_blocks;
use steel_registry::{RegistryExt, REGISTRY};
use steel_utils::{BlockPos, ChunkPos, Identifier, SectionPos};

use crate::entity::next_entity_id;
use crate::worldgen::spawn_rules::{BiomeSpawnRules, SpawnEntry};
use crate::world::World;

const SPAWN_RADIUS: f64 = 24.0;
const MAX_PASSIVE_LIGHT: u8 = 9;
const MAX_HOSTILE_LIGHT: u8 = 7;
const MIN_BAT_Y: i32 = 45;

static ENTITY_KEY_CACHE: std::sync::LazyLock<FxHashMap<&'static str, Identifier>> = 
    std::sync::LazyLock::new(|| {
        let mut map = FxHashMap::default();
        map.insert("bat", Identifier::vanilla_static("bat"));
        map.insert("chicken", Identifier::vanilla_static("chicken"));
        map.insert("cod", Identifier::vanilla_static("cod"));
        map.insert("cow", Identifier::vanilla_static("cow"));
        map.insert("creeper", Identifier::vanilla_static("creeper"));
        map.insert("dolphin", Identifier::vanilla_static("dolphin"));
        map.insert("donkey", Identifier::vanilla_static("donkey"));
        map.insert("enderman", Identifier::vanilla_static("enderman"));
        map.insert("fox", Identifier::vanilla_static("fox"));
        map.insert("horse", Identifier::vanilla_static("horse"));
        map.insert("husk", Identifier::vanilla_static("husk"));
        map.insert("llama", Identifier::vanilla_static("llama"));
        map.insert("mooshroom", Identifier::vanilla_static("mooshroom"));
        map.insert("ocelot", Identifier::vanilla_static("ocelot"));
        map.insert("panda", Identifier::vanilla_static("panda"));
        map.insert("parrot", Identifier::vanilla_static("parrot"));
        map.insert("pig", Identifier::vanilla_static("pig"));
        map.insert("pufferfish", Identifier::vanilla_static("pufferfish"));
        map.insert("rabbit", Identifier::vanilla_static("rabbit"));
        map.insert("salmon", Identifier::vanilla_static("salmon"));
        map.insert("sheep", Identifier::vanilla_static("sheep"));
        map.insert("skeleton", Identifier::vanilla_static("skeleton"));
        map.insert("slime", Identifier::vanilla_static("slime"));
        map.insert("spider", Identifier::vanilla_static("spider"));
        map.insert("squid", Identifier::vanilla_static("squid"));
        map.insert("stray", Identifier::vanilla_static("stray"));
        map.insert("witch", Identifier::vanilla_static("witch"));
        map.insert("wolf", Identifier::vanilla_static("wolf"));
        map.insert("zombie", Identifier::vanilla_static("zombie"));
        map
    });

static SUMMONABLE_ENTITIES: std::sync::LazyLock<std::collections::HashSet<&'static str>> = 
    std::sync::LazyLock::new(|| {
        let mut set = std::collections::HashSet::new();
        set.insert("bat");
        set.insert("blaze");
        set.insert("chicken");
        set.insert("cod");
        set.insert("cow");
        set.insert("creeper");
        set.insert("dolphin");
        set.insert("donkey");
        set.insert("enderman");
        set.insert("fox");
        set.insert("horse");
        set.insert("husk");
        set.insert("llama");
        set.insert("magma_cube");
        set.insert("mooshroom");
        set.insert("ocelot");
        set.insert("panda");
        set.insert("parrot");
        set.insert("pig");
        set.insert("piglin");
        set.insert("pufferfish");
        set.insert("rabbit");
        set.insert("salmon");
        set.insert("sheep");
        set.insert("skeleton");
        set.insert("slime");
        set.insert("spider");
        set.insert("squid");
        set.insert("stray");
        set.insert("witch");
        set.insert("wolf");
        set.insert("zombie");
        set.insert("zombified_piglin");
        set.insert("wither_skeleton");
        set
    });

fn get_entity_type_ref(name: &str) -> Option<EntityTypeRef> {
    use steel_registry::REGISTRY;
    use steel_registry::RegistryExt;
    
    let key = ENTITY_KEY_CACHE.get(name)?;
    REGISTRY.entity_types.by_key(key)
}

const SPAWN_INTERVAL_TICKS: u64 = 400;
const AMBIENT_SPAWN_INTERVAL_TICKS: u64 = 400;
const WATER_SPAWN_INTERVAL_TICKS: u64 = 400;

impl World {
    #[allow(missing_docs)]
    pub fn tick_spawning(self: &Arc<Self>, tick_count: u64) {
        if tick_count % SPAWN_INTERVAL_TICKS != 0 {
            return;
        }

        let Some(closest_player) = self.find_closest_player_chunk() else {
            return;
        };

        self.spawn_passive_mobs(closest_player);
        self.spawn_hostile_mobs(closest_player);

        if tick_count % AMBIENT_SPAWN_INTERVAL_TICKS == 0 {
            self.spawn_ambient_mobs(closest_player);
        }

        if tick_count % WATER_SPAWN_INTERVAL_TICKS == 0 {
            self.spawn_water_mobs(closest_player);
        }
    }

    fn find_closest_player_chunk(&self) -> Option<ChunkPos> {
        let mut rng = rand::rng();
        let mut closest: Option<(ChunkPos, f64)> = None;

        self.players.iter_players(|_, player| {
            let pos = *player.position.lock();
            let chunk_pos = ChunkPos::new(
                SectionPos::block_to_section_coord(pos.x as i32),
                SectionPos::block_to_section_coord(pos.z as i32),
            );

            let dist_sq = pos.x * pos.x + pos.z * pos.z;

            if closest.is_none() || dist_sq < closest.as_ref().unwrap().1 {
                closest = Some((chunk_pos, dist_sq));
            }
            true
        });

        if let Some((closest_chunk, _)) = closest {
            Some(ChunkPos::new(
                closest_chunk.0.x + rng.random_range(-1..=1),
                closest_chunk.0.y + rng.random_range(-1..=1),
            ))
        } else {
            None
        }
    }

    fn is_player_within_radius(&self, pos: &DVec3) -> bool {
        let mut found = false;
        self.players.iter_players(|_, player| {
            let player_pos = *player.position.lock();
            let dist_sq = (player_pos.x - pos.x).powi(2) + (player_pos.z - pos.z).powi(2);
            if dist_sq < SPAWN_RADIUS * SPAWN_RADIUS {
                found = true;
                false
            } else {
                true
            }
        });
        found
    }

    fn get_block_light_level(&self, pos: BlockPos) -> u8 {
        self.chunk_map.with_full_chunk(
            ChunkPos::new(
                SectionPos::block_to_section_coord(pos.0.x),
                SectionPos::block_to_section_coord(pos.0.z),
            ),
            |chunk| {
                let local_x = usize::try_from(pos.0.x & 0xF).unwrap_or(0);
                let local_y = usize::try_from((pos.0.y - chunk.min_y()) & 0xFF).unwrap_or(0);
                let local_z = usize::try_from(pos.0.z & 0xF).unwrap_or(0);

                chunk.sections().get_relative_block(local_x, local_y, local_z)
                    .map(|_| 0u8)
                    .unwrap_or(0)
            },
        ).unwrap_or(0)
    }

    fn get_sky_light_level(&self, _pos: BlockPos) -> u8 {
        0
    }

fn spawn_passive_mobs(self: &Arc<Self>, center_chunk: ChunkPos) {
        let spawn_attempts = 10;
        let mut rng = rand::rng();

        for _ in 0..spawn_attempts {
            let chunk_x = center_chunk.0.x + rng.random_range(-8..=8);
            let chunk_z = center_chunk.0.y + rng.random_range(-8..=8);
            let chunk_pos = ChunkPos::new(chunk_x, chunk_z);

            if let Some(biome_id) = self.get_biome_at(chunk_x * 16 + 8, chunk_z * 16 + 8) {
                let rules = BiomeSpawnRules::get_for_biome(biome_id);
                if rules.creature.is_empty() {
                    continue;
                }

                let (x, z) = self.get_random_position_in_chunk(chunk_pos);
                let surface_y = self.get_surface_height(x, z);

                if surface_y < self.get_min_y() + 1 {
                    continue;
                }

                let spawn_pos = DVec3::new(f64::from(x), f64::from(surface_y), f64::from(z));

                if !self.is_player_within_radius(&spawn_pos) {
                    continue;
                }

                if let Some(entry) = self.select_spawn_entry(&rules.creature) {
                    self.spawn_mob_at(&entry.entity_type, spawn_pos);
                }
            }
        }
    }

    fn spawn_hostile_mobs(self: &Arc<Self>, center_chunk: ChunkPos) {
        let spawn_attempts = 15;
        let mut rng = rand::rng();

        for _ in 0..spawn_attempts {
            let chunk_x = center_chunk.0.x + rng.random_range(-8..=8);
            let chunk_z = center_chunk.0.y + rng.random_range(-8..=8);
            let chunk_pos = ChunkPos::new(chunk_x, chunk_z);

            if let Some(biome_id) = self.get_biome_at(chunk_x * 16 + 8, chunk_z * 16 + 8) {
                let rules = BiomeSpawnRules::get_for_biome(biome_id);
                if rules.monster.is_empty() {
                    continue;
                }

                let (x, z) = self.get_random_position_in_chunk(chunk_pos);
                let surface_y = self.get_surface_height(x, z);
                if surface_y < self.get_min_y() + 1 {
                    continue;
                }
                let spawn_y = surface_y - 1;
                let spawn_pos = DVec3::new(f64::from(x), f64::from(spawn_y), f64::from(z));

                if !self.is_player_within_radius(&spawn_pos) {
                    continue;
                }

                let block_light = self.get_block_light_level(BlockPos::new(x, spawn_y, z));
                let sky_light = self.get_sky_light_level(BlockPos::new(x, spawn_y, z));
                let light_level = block_light.max(sky_light);

                if light_level > MAX_HOSTILE_LIGHT {
                    continue;
                }

                if let Some(entry) = self.select_spawn_entry(&rules.monster) {
                    self.spawn_mob_at(&entry.entity_type, spawn_pos);
                }
            }
        }
    }

    fn spawn_ambient_mobs(self: &Arc<Self>, center_chunk: ChunkPos) {
        let spawn_attempts = 10;
        let mut rng = rand::rng();

        for _ in 0..spawn_attempts {
            let chunk_x = center_chunk.0.x + rng.random_range(-8..=8);
            let chunk_z = center_chunk.0.y + rng.random_range(-8..=8);

            if let Some(biome_id) = self.get_biome_at(chunk_x * 16 + 8, chunk_z * 16 + 8) {
                let rules = BiomeSpawnRules::get_for_biome(biome_id);
                if rules.ambient.is_empty() {
                    continue;
                }

                let (x, z) = {
                    let base_x = chunk_x * 16;
                    let base_z = chunk_z * 16;
                    (
                        base_x + rng.random_range(0..16),
                        base_z + rng.random_range(0..16),
                    )
                };

                let spawn_y = self.get_surface_height(x, z);
                if spawn_y < MIN_BAT_Y {
                    continue;
                }

                let spawn_pos = DVec3::new(f64::from(x), f64::from(spawn_y), f64::from(z));

                if !self.is_player_within_radius(&spawn_pos) {
                    continue;
                }

                let block_light = self.get_block_light_level(BlockPos::new(x, spawn_y, z));
                let sky_light = self.get_sky_light_level(BlockPos::new(x, spawn_y, z));
                let light_level = block_light.max(sky_light);

                if light_level > 0 {
                    continue;
                }

                if let Some(entry) = self.select_spawn_entry(&rules.ambient) {
                    self.spawn_mob_at(&entry.entity_type, spawn_pos);
                }
            }
        }
    }

    fn spawn_water_mobs(self: &Arc<Self>, center_chunk: ChunkPos) {
        let spawn_attempts = 15;
        let mut rng = rand::rng();

        for _ in 0..spawn_attempts {
            let chunk_x = center_chunk.0.x + rng.random_range(-8..=8);
            let chunk_z = center_chunk.0.y + rng.random_range(-8..=8);

            if let Some(biome_id) = self.get_biome_at(chunk_x * 16 + 8, chunk_z * 16 + 8) {
                let rules = BiomeSpawnRules::get_for_biome(biome_id);
                if rules.water_ambient.is_empty() && rules.water_creature.is_empty() {
                    continue;
                }

                let (x, z) = {
                    let base_x = chunk_x * 16;
                    let base_z = chunk_z * 16;
                    (
                        base_x + rng.random_range(0..16),
                        base_z + rng.random_range(0..16),
                    )
                };

                let water_height = self.get_water_height(x, z);
                if water_height < self.get_min_y() + 1 {
                    continue;
                }

                let spawn_pos = DVec3::new(
                    f64::from(x),
                    f64::from(water_height),
                    f64::from(z),
                );

                if !self.is_player_within_radius(&spawn_pos) {
                    continue;
                }

                if !rules.water_ambient.is_empty() {
                    if let Some(entry) = self.select_spawn_entry(&rules.water_ambient) {
                        self.spawn_mob_at(&entry.entity_type, spawn_pos);
                    }
                }

                if !rules.water_creature.is_empty() {
                    if rng.random_range(0..100) < 10 {
                        if let Some(entry) = self.select_spawn_entry(&rules.water_creature) {
                            self.spawn_mob_at(&entry.entity_type, spawn_pos);
                        }
                    }
                }
            }
        }
    }

    fn get_water_height(&self, x: i32, z: i32) -> i32 {
        let max_check = self.get_min_y() + self.get_height() - 2;

        for y in (self.get_min_y()..=max_check).rev() {
            let pos = BlockPos::new(x, y, z);
            let state = self.get_block_state(pos);
            let block = state.get_block();
            if block == &vanilla_blocks::WATER {
                return y;
            }
        }

        self.get_min_y()
    }

    fn get_random_position_in_chunk(&self, chunk_pos: ChunkPos) -> (i32, i32) {
        let base_x = chunk_pos.0.x * 16;
        let base_z = chunk_pos.0.y * 16;
        let mut rng = rand::rng();
        let x = base_x + rng.random_range(0..16);
        let z = base_z + rng.random_range(0..16);
        (x, z)
    }

    fn get_surface_height(&self, x: i32, z: i32) -> i32 {
        let check_y = (self.get_min_y() + self.get_height() - 2).min(self.get_max_y());
        
        for y in (self.get_min_y()..=check_y).rev() {
            let pos = BlockPos::new(x, y, z);
            let state = self.get_block_state(pos);
            if state.is_solid() {
                return y + 1;
            }
        }
        
        self.get_min_y() + 50
    }

    fn find_valid_spawn_position(
        self: &Arc<Self>,
        x: i32,
        surface_y: i32,
        z: i32,
        category: SpawnCategory,
    ) -> Option<DVec3> {
        let spawn_y = if category == SpawnCategory::Hostile {
            surface_y - 1
        } else {
            surface_y
        };

        let block_pos = BlockPos::new(x, spawn_y - 1, z);

        let ground_state = self.get_block_state(block_pos);
        if ground_state.is_solid() {
            let spawn_block_state = self.get_block_state(BlockPos::new(x, spawn_y, z));
            let block_at_spawn = spawn_block_state.get_block();

            if block_at_spawn == &vanilla_blocks::AIR
                || block_at_spawn == &vanilla_blocks::WATER
                || block_at_spawn == &vanilla_blocks::LAVA
            {
                return Some(DVec3::new(
                    f64::from(x) + 0.5,
                    f64::from(spawn_y),
                    f64::from(z) + 0.5,
                ));
            }
        }

        None
    }

    fn select_spawn_entry<'a>(&self, entries: &'a [SpawnEntry]) -> Option<&'a SpawnEntry> {
        if entries.is_empty() {
            return None;
        }

        let total_weight: i32 = entries.iter().map(|e| e.weight).sum();
        if total_weight <= 0 {
            return Some(&entries[0]);
        }

        let mut rng = rand::rng();
        let mut roll = rng.random_range(0..total_weight);

        for entry in entries {
            roll -= entry.weight;
            if roll < 0 {
                return Some(entry);
            }
        }

entries.first()
    }

    pub fn spawn_mob_at(self: &Arc<Self>, entity_type: &str, pos: DVec3) {
        let name = if entity_type.contains(':') {
            &entity_type[9..]
        } else {
            entity_type
        };

        if !SUMMONABLE_ENTITIES.contains(name) {
            return;
        }

        let block_x = pos.x as i32;
        let block_z = pos.z as i32;
        let biome_id = self.get_biome_at(block_x, block_z);

        if !self.check_biome_restrictions(name, biome_id, pos) {
            return;
        }

        use crate::entity::entities::MobEntity;

        let entity_type_ref = get_entity_type_ref(name);

        if let Some(et) = entity_type_ref {
            let entity_id = next_entity_id();
            let entity = Arc::new(MobEntity::new(et, entity_id, pos, Arc::downgrade(self)));
            self.add_entity(entity);
        }
    }

    fn check_biome_restrictions(&self, entity_type: &str, biome_id: Option<u16>, pos: DVec3) -> bool {
        let Some(biome_id) = biome_id else {
            return false;
        };

        let biome_key = REGISTRY.biomes.by_id(biome_id as usize)
            .map(|b| b.key.path.as_ref())
            .unwrap_or("");

        match entity_type {
            "enderman" => {
                biome_key.starts_with("the_end") || biome_key.starts_with("end_") || biome_key == "end_void"
            }
            "blaze" | "piglin" => {
                biome_key.starts_with("nether") || biome_key.contains("nether")
            }
            "stray" => {
                biome_key.contains("snowy") || biome_key.contains("ice") || biome_key == "frozen_river"
            }
            "husk" => {
                biome_key == "desert" || biome_key == "desert_lakes"
            }
            "slime" => {
                biome_key == "swamp" || biome_key == "mangrove_swamp"
            }
            _ => true,
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum SpawnCategory {
    Passive,
    Hostile,
}

impl SpawnCategory {
    pub(crate) fn is_hostile(&self) -> bool {
        matches!(self, SpawnCategory::Hostile)
    }
}
