//! Jigsaw structure generation module.
//!
//! Implements vanilla jigsaw structure generation by reading:
//! - Template pool JSON files defining piece pools
//! - NBT files containing piece geometry
//! - Jigsaw connections for piece-to-piece linking

use std::sync::OnceLock;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use steel_utils::Identifier;

#[derive(Debug, Clone, Deserialize)]
pub struct TemplatePool {
    pub elements: Vec<JigsawPiece>,
    #[serde(rename = "fallback")]
    pub fallback: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JigsawPiece {
    #[serde(rename = "element")]
    pub element: JigsawPieceInner,
    pub weight: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JigsawPieceInner {
    #[serde(rename = "element_type")]
    pub element_type: String,
    pub location: String,
    pub processors: Option<serde_json::Value>,
    pub projection: String,
    #[serde(rename = "nbt")]
    pub nbt_data: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JigsawConnection {
    pub name: String,
    pub target: String,
    pub pool: String,
    pub turn: String,
}

#[derive(Debug, Clone)]
pub struct PoolEntry {
    pub location: String,
    pub processors: Option<String>,
    pub projection: Projection,
    pub weight: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    Rigid,
    TerrainMatching,
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Rigid
    }
}

impl From<&str> for Projection {
    fn from(s: &str) -> Self {
        match s {
            "terrain_matching" => Projection::TerrainMatching,
            _ => Projection::Rigid,
        }
    }
}

static TEMPLATE_POOLS: OnceLock<FxHashMap<Identifier, TemplatePool>> = OnceLock::new();

pub fn load_pool(pool_id: &Identifier) -> Option<&'static TemplatePool> {
    let pools = TEMPLATE_POOLS.get_or_init(|| {
        let mut m = FxHashMap::default();
        let base_path = "steel-registry/build_assets/builtin_datapacks/minecraft/worldgen/template_pool";
        if let Ok(base_dir) = std::fs::read_dir(base_path) {
            for dir_entry in base_dir.flatten() {
                let dir_path = dir_entry.path();
                if dir_path.is_dir() {
                    if let Ok(subdir) = std::fs::read_dir(&dir_path) {
                        for entry in subdir.flatten() {
                            let path = entry.path();
                            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    if let Ok(pool) = serde_json::from_str::<TemplatePool>(&content) {
                                        let pool_name = path.file_stem().unwrap().to_str().unwrap();
                                        let parent_name = path.parent().and_then(|p| p.file_name()).unwrap().to_str().unwrap();
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