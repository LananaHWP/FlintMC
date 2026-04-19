use std::sync::OnceLock;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use steel_registry::{REGISTRY, RegistryExt};
use steel_utils::Identifier;
use steel_utils::random::xoroshiro::Xoroshiro;
use steel_utils::random::Random;
use steel_utils::noise::ImprovedNoise;

use crate::chunk::chunk_access::ChunkAccess;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfiguredCarver {
    #[serde(rename = "type")]
    pub carver_type: String,
    pub config: CarverConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarverConfig {
    pub probability: f32,
    pub y: HeightRange,
    pub y_scale: Option<f32>,
    pub horizontal_radius_multiplier: Option<HorizontalMultiplier>,
    pub vertical_radius_multiplier: Option<HorizontalMultiplier>,
    pub lava_level: Option<LavaLevel>,
    pub replaceable: String,
    pub floor_level: Option<f32>,
    pub shape: Option<CarverShape>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum HeightRange {
    Uniform {
        #[serde(rename = "type")]
        range_type: String,
        max_inclusive: MaxInclusive,
        min_inclusive: MinInclusive,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MaxInclusive {
    Absolute { absolute: i32 },
    AboveBottom { above_bottom: i32 },
    BelowTop { below_top: i32 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MinInclusive {
    Absolute { absolute: i32 },
    AboveBottom { above_bottom: i32 },
    BelowTop { below_top: i32 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum HorizontalMultiplier {
    Uniform {
        #[serde(rename = "type")]
        range_type: String,
        max_exclusive: f32,
        min_inclusive: f32,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LavaLevel {
    AboveBottom { above_bottom: i32 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarverShape {
    pub thickness: Thickness,
    pub width_smoothness: Option<i32>,
    pub distance_factor: Option<HorizontalMultiplier>,
    pub horizontal_radius_factor: Option<HorizontalMultiplier>,
    pub vertical_radius_center_factor: Option<f32>,
    pub vertical_radius_default_factor: Option<f32>,
    pub vertical_rotation: Option<HorizontalMultiplier>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Thickness {
    Trapezoid {
        #[serde(rename = "type")]
        range_type: String,
        max: f32,
        min: f32,
        plateau: f32,
    },
}

static CONFIGURED_CARVERS: OnceLock<FxHashMap<Identifier, ConfiguredCarver>> = OnceLock::new();

pub fn get_configured_carver(name: &Identifier) -> Option<&'static ConfiguredCarver> {
    let map = CONFIGURED_CARVERS.get_or_init(|| {
        let mut m = FxHashMap::default();
        if let Ok(dir) = std::fs::read_dir("steel-registry/build_assets/builtin_datapacks/minecraft/worldgen/configured_carver") {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<ConfiguredCarver>(&content) {
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

pub struct CarverGenerator {
    seed: u64,
    random: Xoroshiro,
    carver_noise: ImprovedNoise,
    carver_noise2: ImprovedNoise,
}

impl CarverGenerator {
    pub fn new(seed: u64) -> Self {
        let mut random1 = Xoroshiro::from_seed(seed);
        let mut random2 = Xoroshiro::from_seed(seed + 1000);
        Self {
            seed,
            random: Xoroshiro::from_seed(seed),
            carver_noise: ImprovedNoise::new(&mut random1),
            carver_noise2: ImprovedNoise::new(&mut random2),
        }
    }

    pub fn apply_biome_carvers(&mut self, chunk: &ChunkAccess) {
        let pos = chunk.pos();
        let chunk_min_x = pos.0.x * 16;
        let chunk_min_z = pos.0.y * 16;
        let min_y = chunk.min_y();
        let height = chunk.height();

        let get_biome = |qx: i32, qz: i32| -> u16 {
            let biome_data = chunk.sections().read_all_biomes();
            let section_idx = 0usize.min(chunk.sections().sections.len().saturating_sub(1));
            let local_qx = ((qx & 3) + 4) as usize;
            let local_qz = ((qz & 3) + 4) as usize;
            biome_data.get(section_idx * 64 + local_qz * 4 + local_qx).copied().unwrap_or(0)
        };

        for local_x in 0..16i32 {
            for local_z in 0..16i32 {
                let world_x = chunk_min_x + local_x;
                let world_z = chunk_min_z + local_z;

                let biome_quart_x = world_x >> 2;
                let biome_quart_z = world_z >> 2;
                let biome_id = get_biome(biome_quart_x, biome_quart_z);

                if let Some(biome) = REGISTRY.biomes.by_id(biome_id as usize) {
                    for carver_id in &biome.carvers {
                        self.carve_carver(chunk, carver_id, world_x, world_z, min_y, height);
                    }
                }
            }
        }

        chunk.mark_dirty();
    }

    fn carve_carver(&mut self, chunk: &ChunkAccess, carver_id: &Identifier, world_x: i32, world_z: i32, min_y: i32, height: i32) {
        let Some(carver) = get_configured_carver(carver_id) else {
            return;
        };

        if self.random.next_f32() > carver.config.probability {
            return;
        }

        let local_x = (world_x % 16).abs() as usize;
        let local_z = (world_z % 16).abs() as usize;

        let surface_y = chunk.get_surface_height(local_x, local_z).unwrap_or(64);

        let (min_y_level, _max_y_level) = match &carver.config.y {
            HeightRange::Uniform { range_type: _, max_inclusive, min_inclusive } => {
                let max_y = match max_inclusive {
                    MaxInclusive::Absolute { absolute } => *absolute,
                    MaxInclusive::AboveBottom { above_bottom } => min_y + height - above_bottom,
                    MaxInclusive::BelowTop { below_top } => height - below_top,
                };
                let min_y_val = match min_inclusive {
                    MinInclusive::Absolute { absolute } => *absolute,
                    MinInclusive::AboveBottom { above_bottom } => min_y + above_bottom,
                    MinInclusive::BelowTop { below_top } => height - below_top,
                };
                (min_y_val, max_y)
            }
        };

        let y_scale = carver.config.y_scale.unwrap_or(1.0) as f64;

        let (horizontal_radius, vertical_radius) = self.get_radii(
            carver.config.horizontal_radius_multiplier.as_ref(),
            carver.config.vertical_radius_multiplier.as_ref(),
            world_x,
            world_z,
        );

        let carve_start = (surface_y as f32 - vertical_radius as f32).max(min_y_level as f32) as i32;
        let carve_end = surface_y;

        if carve_start >= carve_end || horizontal_radius < 2 {
            return;
        }

        let nodes = self.generate_cave_nodes(
            world_x,
            world_z,
            carve_start,
            carve_end,
            horizontal_radius,
            y_scale,
        );

        if nodes.is_empty() {
            return;
        }

        for node in &nodes {
            self.carve_node(
                chunk,
                node,
                min_y,
                height,
            );
        }

        if let Some(lava_level) = &carver.config.lava_level {
            self.place_lava(chunk, &nodes, min_y, height, lava_level);
        }
    }

    fn get_radii(
        &mut self,
        horizontal_multiplier: Option<&HorizontalMultiplier>,
        vertical_multiplier: Option<&HorizontalMultiplier>,
        world_x: i32,
        world_z: i32,
    ) -> (i32, i32) {
        let h_radius = match horizontal_multiplier {
            Some(HorizontalMultiplier::Uniform { range_type: _, max_exclusive, min_inclusive }) => {
                let h = self.get_noise_value(world_x, world_z, *min_inclusive as f64, *max_exclusive as f64);
                h as i32
            }
            None => 12,
        };

        let v_radius = match vertical_multiplier {
            Some(HorizontalMultiplier::Uniform { range_type: _, max_exclusive, min_inclusive }) => {
                let v = self.get_noise_value(world_x, world_z, *min_inclusive as f64, *max_exclusive as f64);
                v as i32
            }
            None => 8,
        };

        (h_radius, v_radius)
    }

    fn get_noise_value(&mut self, x: i32, z: i32, min: f64, max: f64) -> f64 {
        let noise_val = self.carver_noise.noise(
            x as f64 * 0.1,
            0.0,
            z as f64 * 0.1,
        );
        let normalized = (noise_val + 1.0) * 0.5;
        min + normalized * (max - min)
    }

    fn generate_cave_nodes(
        &mut self,
        world_x: i32,
        world_z: i32,
        carve_start: i32,
        carve_end: i32,
        max_radius: i32,
        y_scale: f64,
    ) -> Vec<CaveNode> {
        let mut nodes = Vec::new();
        let mut current_y = carve_start;
        let mut angle = self.random.next_f32() * std::f32::consts::TAU;
        let mut distance = self.random.next_f32() * 8.0 + 4.5;
        let mut y_incline = (self.random.next_f32() - 0.5) * y_scale as f32;

        let (thickness, plateau) = self.get_thickness_params();

        while current_y < carve_end - 2 {
            let dx = (angle.cos() * distance) as i32;
            let dz = (angle.sin() * distance) as i32;
            let dy = (y_incline * distance) as i32;

            let radius = max_radius.max(2);

            nodes.push(CaveNode {
                x: world_x + dx,
                y: current_y + dy,
                z: world_z + dz,
                radius,
                thickness,
                plateau,
            });

            current_y += self.random.next_i32() % 3 + 1;
            distance = self.random.next_f32() * 8.0 + 4.5;
            angle += (self.random.next_f32() - 0.5) * 0.5;
            y_incline = (self.random.next_f32() - 0.5) * y_scale as f32;
        }

        nodes
    }

    fn get_thickness_params(&mut self) -> (i32, i32) {
        let max_t = self.random.next_f32() * 1.5 + 1.0;
        let min_t = self.random.next_f32() * 1.5 + 1.0;
        let plateau = self.random.next_f32() * 2.0;

        (max_t.min(min_t) as i32, plateau as i32)
    }

    fn carve_node(
        &mut self,
        chunk: &ChunkAccess,
        node: &CaveNode,
        min_y: i32,
        height: i32,
    ) {
        let radius_squared = node.radius as f64 * node.radius as f64;

        let node_min_x = (node.x - node.radius).max(chunk.pos().0.x * 16);
        let node_max_x = (node.x + node.radius).min(chunk.pos().0.x * 16 + 15);
        let node_min_z = (node.z - node.radius).max(chunk.pos().0.y * 16);
        let node_max_z = (node.z + node.radius).min(chunk.pos().0.y * 16 + 15);

        for lx in node_min_x..=node_max_x {
            for lz in node_min_z..=node_max_z {
                let dx = lx - node.x;
                let dz = lz - node.z;

                let local_x = (lx % 16).abs() as usize;
                let local_z = (lz % 16).abs() as usize;

                let dist_squared = (dx * dx + dz * dz) as f64;
                if dist_squared < radius_squared {
                    self.carve_horizontal_slice(
                        chunk,
                        local_x,
                        local_z,
                        node,
                        min_y,
                        height,
                    );
                }
            }
        }
    }

    fn carve_horizontal_slice(
        &mut self,
        chunk: &ChunkAccess,
        local_x: usize,
        local_z: usize,
        node: &CaveNode,
        min_y: i32,
        height: i32,
    ) {
        let node_min_y = (node.y - node.radius).max(min_y);
        let node_max_y = (node.y + node.radius).min(min_y + height - 1);

        for ly in node_min_y..=node_max_y {
            let dy = ly - node.y;
            let dist_from_center = dy.abs();

            if dist_from_center > node.radius {
                continue;
            }

            let relative_y = (ly - min_y) as usize;
            if relative_y < height as usize {
                if let Some(current_block) = chunk.get_relative_block(local_x, relative_y, local_z) {
                    let air_id = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::AIR);
                    chunk.set_relative_block(local_x, relative_y, local_z, air_id);
                }
            }
        }
    }

    fn place_lava(
        &mut self,
        chunk: &ChunkAccess,
        nodes: &[CaveNode],
        min_y: i32,
        height: i32,
        lava_level: &LavaLevel,
    ) {
        let lava_y = match lava_level {
            LavaLevel::AboveBottom { above_bottom } => min_y + above_bottom,
        };

        let local_x = (nodes[0].x % 16).abs() as usize;
        let local_z = (nodes[0].z % 16).abs() as usize;

        let relative_y = (lava_y - min_y) as usize;
        if relative_y < height as usize {
            let air_id = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::AIR);
            if let Some(current_block) = chunk.get_relative_block(local_x, relative_y, local_z) {
                if Some(air_id) == Some(current_block) {
                    let lava_id = REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::LAVA);
                    chunk.set_relative_block(local_x, relative_y, local_z, lava_id);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CaveNode {
    x: i32,
    y: i32,
    z: i32,
    radius: i32,
    thickness: i32,
    plateau: i32,
}