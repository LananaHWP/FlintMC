//! Mob spawning rules for vanilla biome-based spawning.
//!
//! Implements biome spawn rules for:
//! - creature (passive mobs)
//! - monster (-hostile mobs)
//! - ambient (bats)
//! - water_ambient (fish)
//! - water_creature (dolphins, guardians)

use steel_registry::{REGISTRY, RegistryExt};
use std::sync::OnceLock;

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct SpawnEntry {
    pub entity_type: String,
    pub weight: i32,
    pub min_count: i32,
    pub max_count: i32,
}

#[derive(Debug, Clone, Default)]
pub struct BiomeSpawnRules {
    pub creature: Vec<SpawnEntry>,
    pub monster: Vec<SpawnEntry>,
    pub ambient: Vec<SpawnEntry>,
    pub water_ambient: Vec<SpawnEntry>,
    pub water_creature: Vec<SpawnEntry>,
}

impl BiomeSpawnRules {
    pub fn get_for_biome(biome_id: u16) -> &'static Self {
        let key = if let Some(biome) = REGISTRY.biomes.by_id(biome_id as usize) {
            biome.key.path.as_ref()
        } else {
            "plains"
        };
        
        match key {
            // Nether biomes
            "nether_wastes" | "soul_sand_valley" | "crimson_forest" | "warped_forest" | "basalt_deltas" | "the_nether" => Self::nether(),
            
            // End biomes
            "the_end" | "end_midlands" | "end_highlands" | "end_barrens" | "small_end_islands" | "end_void" => Self::the_end(),
            
            // Overworld biomes
            "desert" => Self::desert(),
            "forest" | "flower_forest" | "birch_forest" => Self::forest(),
            "taiga" | "old_growth_taiga" | "giant_tree_taiga" | "giant_spruce_taiga" => Self::taiga(),
            "snowy_taiga" | "snowy_tundra" => Self::snowy(),
            "jungle" | "sparse_jungle" | "jungle_edge" | "modified_jungle" | "modified_jungle_edge" => Self::jungle(),
            "savanna" | "windswept_savanna" => Self::savanna(),
            "swamp" | "mangrove_swamp" => Self::swamp(),
            "ocean" | "deep_ocean" | "cold_ocean" | "warm_ocean" | "lukewarm_ocean" | "frozen_ocean" | "deep_frozen_ocean" | "deep_cold_ocean" | "deep_lukewarm_ocean" => Self::ocean(),
            "river" | "frozen_river" => Self::river(),
            "mushroom_fields" | "mushroom_field_shore" => Self::mushroom_island(),
            "badlands" | "eroded_badlands" | "wooded_badlands" | "badlands_plateau" | "modified_badlands_plateau" => Self::badlands(),
            "plains" | "sunflower_plains" => Self::plains(),
            "snowy_plains" => Self::snowy(),
            "mountains" | "gravelly_mountains" | "mountain_edge" => Self::mountains(),
            "wooded_mountains" => Self::wooded_mountains(),
            "savanna_plateau" | "windswept_hills" | "windswept_gravelly_hills" => Self::windswept(),
            "dark_forest" => Self::dark_forest(),
            "desert_lakes" => Self::desert(),
            _ => Self::plains(),
        }
    }
    
    fn plains() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "sheep".into(), weight: 12, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "pig".into(), weight: 10, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "chicken".into(), weight: 10, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "cow".into(), weight: 8, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "horse".into(), weight: 5, min_count: 2, max_count: 6 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "creeper".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "spider".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "slime".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "enderman".into(), weight: 10, min_count: 1, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn desert() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "rabbit".into(), weight: 4, min_count: 2, max_count: 3 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "husk".into(), weight: 80, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "zombie".into(), weight: 19, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 80, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "creeper".into(), weight: 80, min_count: 4, max_count: 4 },
            ],
            ambient: vec![],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn forest() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "sheep".into(), weight: 12, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "pig".into(), weight: 10, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "chicken".into(), weight: 10, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "cow".into(), weight: 8, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "wolf".into(), weight: 5, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "creeper".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "spider".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn taiga() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "rabbit".into(), weight: 4, min_count: 2, max_count: 3 },
                SpawnEntry { entity_type: "fox".into(), weight: 8, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn snowy() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "rabbit".into(), weight: 4, min_count: 2, max_count: 3 },
                SpawnEntry { entity_type: "fox".into(), weight: 8, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "stray".into(), weight: 80, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 80, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn jungle() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "parrot".into(), weight: 10, min_count: 1, max_count: 2 },
                SpawnEntry { entity_type: "panda".into(), weight: 5, min_count: 1, max_count: 2 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "creeper".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "ocelot".into(), weight: 2, min_count: 1, max_count: 1 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn savanna() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "sheep".into(), weight: 12, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "donkey".into(), weight: 1, min_count: 1, max_count: 3 },
                SpawnEntry { entity_type: "horse".into(), weight: 5, min_count: 2, max_count: 6 },
                SpawnEntry { entity_type: "llama".into(), weight: 8, min_count: 4, max_count: 6 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn swamp() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "slime".into(), weight: 10, min_count: 4, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "witch".into(), weight: 5, min_count: 1, max_count: 1 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn ocean() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![],
            monster: vec![],
            ambient: vec![],
            water_ambient: vec![
                SpawnEntry { entity_type: "cod".into(), weight: 10, min_count: 3, max_count: 5 },
                SpawnEntry { entity_type: "pufferfish".into(), weight: 3, min_count: 1, max_count: 2 },
            ],
            water_creature: vec![
                SpawnEntry { entity_type: "dolphin".into(), weight: 5, min_count: 1, max_count: 2 },
                SpawnEntry { entity_type: "squid".into(), weight: 3, min_count: 1, max_count: 2 },
            ],
        })
    }
    
    fn river() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![],
            monster: vec![],
            ambient: vec![],
            water_ambient: vec![
                SpawnEntry { entity_type: "cod".into(), weight: 10, min_count: 3, max_count: 5 },
                SpawnEntry { entity_type: "salmon".into(), weight: 5, min_count: 1, max_count: 5 },
            ],
            water_creature: vec![
                SpawnEntry { entity_type: "dolphin".into(), weight: 2, min_count: 1, max_count: 2 },
            ],
        })
    }
    
    fn mushroom_island() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "mooshroom".into(), weight: 8, min_count: 4, max_count: 8 },
            ],
            monster: vec![],
            ambient: vec![],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn badlands() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "rabbit".into(), weight: 4, min_count: 2, max_count: 3 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 80, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn mountains() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "goat".into(), weight: 5, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn wooded_mountains() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "sheep".into(), weight: 12, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "goat".into(), weight: 5, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn windswept() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "sheep".into(), weight: 12, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "goat".into(), weight: 5, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn dark_forest() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![
                SpawnEntry { entity_type: "sheep".into(), weight: 12, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "pig".into(), weight: 10, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "chicken".into(), weight: 10, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "cow".into(), weight: 8, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "wolf".into(), weight: 5, min_count: 2, max_count: 4 },
            ],
            monster: vec![
                SpawnEntry { entity_type: "zombie".into(), weight: 95, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "creeper".into(), weight: 100, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "spider".into(), weight: 100, min_count: 4, max_count: 4 },
            ],
            ambient: vec![
                SpawnEntry { entity_type: "bat".into(), weight: 10, min_count: 8, max_count: 8 },
            ],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn nether() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![],
            monster: vec![
                SpawnEntry { entity_type: "blaze".into(), weight: 10, min_count: 2, max_count: 3 },
                SpawnEntry { entity_type: "zombified_piglin".into(), weight: 5, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "wither_skeleton".into(), weight: 8, min_count: 5, max_count: 5 },
                SpawnEntry { entity_type: "skeleton".into(), weight: 2, min_count: 5, max_count: 5 },
                SpawnEntry { entity_type: "magma_cube".into(), weight: 3, min_count: 4, max_count: 4 },
                SpawnEntry { entity_type: "piglin".into(), weight: 10, min_count: 4, max_count: 4 },
            ],
            ambient: vec![],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
    
    fn the_end() -> &'static Self {
        static RULES: OnceLock<BiomeSpawnRules> = OnceLock::new();
        RULES.get_or_init(|| BiomeSpawnRules {
            creature: vec![],
            monster: vec![
                SpawnEntry { entity_type: "enderman".into(), weight: 10, min_count: 1, max_count: 4 },
            ],
            ambient: vec![],
            water_ambient: vec![],
            water_creature: vec![],
        })
    }
}