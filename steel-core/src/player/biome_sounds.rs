use std::sync::{atomic::{AtomicU32, Ordering}, Arc};

use glam::DVec3;
use steel_utils::{BlockPos, locks::SyncMutex};
use steel_protocol::packets::game::{CSound, SoundSource};
use steel_protocol::packet_traits::EncodedPacket;
use steel_protocol::utils::ConnectionProtocol;
use steel_registry::{REGISTRY, RegistryExt};
use steel_registry::sound_events;

use crate::config::STEEL_CONFIG;
use crate::world::World;
use crate::player::connection::NetworkConnection;

use super::Player;

const AMBIENT_TICK_INTERVAL: u32 = 20;
const MOOD_TICK_INTERVAL: u32 = 1;

pub struct BiomeAmbientState {
    last_biome_id: Option<u16>,
    ambient_sound_counter: AtomicU32,
    mood_sound_counter: AtomicU32,
    additions_sound_counter: AtomicU32,
    last_mood_pos: SyncMutex<Option<BlockPos>>,
}

impl BiomeAmbientState {
    pub fn new() -> Self {
        Self {
            last_biome_id: None,
            ambient_sound_counter: AtomicU32::new(0),
            mood_sound_counter: AtomicU32::new(0),
            additions_sound_counter: AtomicU32::new(0),
            last_mood_pos: SyncMutex::new(None),
        }
    }
}

impl Default for BiomeAmbientState {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn tick_biome_ambient(player: &Player, world: &Arc<World>) {
    let mut state = player.biome_ambient_state.lock();
    let pos = *player.position.lock();

    let block_x = pos.x as i32;
    let block_z = pos.z as i32;
    let block_y = pos.y as i32;

    let Some(biome_id) = world.get_biome_at(block_x, block_z) else {
        return;
    };

    if let Some(biome) = REGISTRY.biomes.by_id(biome_id as usize) {
        let effects = &biome.effects;

        let ambient_counter = state.ambient_sound_counter.fetch_add(1, Ordering::Relaxed);
        let should_play_ambient = ambient_counter >= AMBIENT_TICK_INTERVAL;
        if should_play_ambient {
            state.ambient_sound_counter.store(0, Ordering::Relaxed);
        }

        if Some(biome_id) != state.last_biome_id || should_play_ambient {
            state.last_biome_id = Some(biome_id);

            if let Some(ambient_sound) = &effects.ambient_sound {
                if should_play_ambient {
                    let sound_id = get_sound_id(ambient_sound);
                    if let Some(sound_id) = sound_id {
                        play_ambient_sound(player, sound_id, pos);
                    }
                }
            }
        }

        let mood_counter = state.mood_sound_counter.fetch_add(1, Ordering::Relaxed);
        if mood_counter >= MOOD_TICK_INTERVAL {
            state.mood_sound_counter.store(0, Ordering::Relaxed);
        }

        if let Some(mood_sound) = &effects.mood_sound {
            if mood_counter >= MOOD_TICK_INTERVAL {
                let tick_delay = mood_sound.tick_delay;
                let block_search_extent = mood_sound.block_search_extent;
                let offset = mood_sound.offset;

                let current_tick = world.get_world_tick();
                let offset_ticks = ((offset * 20.0) as i64).rem_euclid(tick_delay as i64);
                let adjusted_tick = (current_tick as i64 + offset_ticks) % tick_delay as i64;
                if adjusted_tick == 0 {
                    let player_block_pos = BlockPos::new(block_x, block_y, block_z);

                    let mut last_pos_guard = state.last_mood_pos.lock();
                    let last_pos = *last_pos_guard;

                    if last_pos.is_none()
                        || last_pos
                            .map(|lp| {
                                let dx = lp.x() - player_block_pos.x();
                                let dz = lp.z() - player_block_pos.z();
                                (dx * dx + dz * dz) as i32
                                    > block_search_extent * block_search_extent
                            })
                            .unwrap_or(true)
                    {
                        let mood_sound_id = get_sound_id(&mood_sound.sound);
                        if let Some(sound_id) = mood_sound_id {
                            play_mood_sound(player, sound_id, pos);
                            *last_pos_guard = Some(player_block_pos);
                        }
                    }
                }
            }
        }

        let additions_counter = state.additions_sound_counter.fetch_add(1, Ordering::Relaxed);
        if additions_counter >= MOOD_TICK_INTERVAL {
            state.additions_sound_counter.store(0, Ordering::Relaxed);
        }

        if let Some(additions_sound) = &effects.additions_sound {
            let roll = rand::random::<f64>();
            if roll < additions_sound.tick_chance {
                let sound_id = get_sound_id(&additions_sound.sound);
                if let Some(sound_id) = sound_id {
                    play_additions_sound(player, sound_id, pos);
                }
            }
        }
    }
}

fn get_sound_id(identifier: &steel_utils::Identifier) -> Option<i32> {
    let path = identifier.path.to_string();
    let sound_name = path.to_uppercase().replace('.', "_");

    match sound_name.as_str() {
        "AMBIENT_FOREST_ADDITIONS" => Some(sound_events::AMBIENT_BASALT_DELTAS_ADDITIONS),
        "AMBIENT_FOREST_LOOP" => Some(sound_events::AMBIENT_BASALT_DELTAS_LOOP),
        "AMBIENT_FOREST_MOOD" => Some(sound_events::AMBIENT_BASALT_DELTAS_MOOD),
        "AMBIENT_CAVE" => Some(sound_events::AMBIENT_CAVE),
        "AMBIENT_UNDERWATER_ENTER" => Some(sound_events::AMBIENT_UNDERWATER_ENTER),
        "AMBIENT_UNDERWATER_EXIT" => Some(sound_events::AMBIENT_UNDERWATER_EXIT),
        "AMBIENT_UNDERWATER_LOOP" => Some(sound_events::AMBIENT_UNDERWATER_LOOP),
        "AMBIENT_UNDERWATER_LOOP_ADDITIONS" => Some(sound_events::AMBIENT_UNDERWATER_LOOP_ADDITIONS),
        "AMBIENT_UNDERWATER_LOOP_ADDITIONS_RARE" => Some(sound_events::AMBIENT_UNDERWATER_LOOP_ADDITIONS_RARE),
        "AMBIENT_UNDERWATER_LOOP_ADDITIONS_ULTRA_RARE" => Some(sound_events::AMBIENT_UNDERWATER_LOOP_ADDITIONS_ULTRA_RARE),
        "AMBIENT_NETHER_WASTES_ADDITIONS" => Some(sound_events::AMBIENT_NETHER_WASTES_ADDITIONS),
        "AMBIENT_NETHER_WASTES_LOOP" => Some(sound_events::AMBIENT_NETHER_WASTES_LOOP),
        "AMBIENT_NETHER_WASTES_MOOD" => Some(sound_events::AMBIENT_NETHER_WASTES_MOOD),
        "AMBIENT_CRIMSON_FOREST_ADDITIONS" => Some(sound_events::AMBIENT_CRIMSON_FOREST_ADDITIONS),
        "AMBIENT_CRIMSON_FOREST_LOOP" => Some(sound_events::AMBIENT_CRIMSON_FOREST_LOOP),
        "AMBIENT_CRIMSON_FOREST_MOOD" => Some(sound_events::AMBIENT_CRIMSON_FOREST_MOOD),
        "AMBIENT_WARPED_FOREST_ADDITIONS" => Some(sound_events::AMBIENT_WARPED_FOREST_ADDITIONS),
        "AMBIENT_WARPED_FOREST_LOOP" => Some(sound_events::AMBIENT_WARPED_FOREST_LOOP),
        "AMBIENT_WARPED_FOREST_MOOD" => Some(sound_events::AMBIENT_WARPED_FOREST_MOOD),
        "AMBIENT_SOUL_SAND_VALLEY_ADDITIONS" => Some(sound_events::AMBIENT_SOUL_SAND_VALLEY_ADDITIONS),
        "AMBIENT_SOUL_SAND_VALLEY_LOOP" => Some(sound_events::AMBIENT_SOUL_SAND_VALLEY_LOOP),
        "AMBIENT_SOUL_SAND_VALLEY_MOOD" => Some(sound_events::AMBIENT_SOUL_SAND_VALLEY_MOOD),
        "AMBIENT_BASALT_DELTAS_ADDITIONS" => Some(sound_events::AMBIENT_BASALT_DELTAS_ADDITIONS),
        "AMBIENT_BASALT_DELTAS_LOOP" => Some(sound_events::AMBIENT_BASALT_DELTAS_LOOP),
        "AMBIENT_BASALT_DELTAS_MOOD" => Some(sound_events::AMBIENT_BASALT_DELTAS_MOOD),
        "WEATHER_RAIN_ABOVE" => None,
        _ => {
            log::debug!("Unknown ambient sound: {}", sound_name);
            None
        }
    }
}

fn play_ambient_sound(player: &Player, sound_id: i32, pos: DVec3) {
    let seed = rand::random::<i64>();
    let packet = CSound::new(
        sound_id,
        SoundSource::Ambient,
        pos.x,
        pos.y,
        pos.z,
        1.0,
        1.0,
        seed,
    );

    if let Ok(encoded) =
        EncodedPacket::from_bare(packet, STEEL_CONFIG.compression, ConnectionProtocol::Play)
    {
        player.connection.send_encoded(encoded);
    }
}

fn play_mood_sound(player: &Player, sound_id: i32, pos: DVec3) {
    let seed = rand::random::<i64>();
    let packet = CSound::new(
        sound_id,
        SoundSource::Ambient,
        pos.x,
        pos.y,
        pos.z,
        0.5,
        1.0,
        seed,
    );

    if let Ok(encoded) =
        EncodedPacket::from_bare(packet, STEEL_CONFIG.compression, ConnectionProtocol::Play)
    {
        player.connection.send_encoded(encoded);
    }
}

fn play_additions_sound(player: &Player, sound_id: i32, pos: DVec3) {
    let seed = rand::random::<i64>();
    let packet = CSound::new(
        sound_id,
        SoundSource::Ambient,
        pos.x,
        pos.y,
        pos.z,
        1.0,
        1.0,
        seed,
    );

    if let Ok(encoded) =
        EncodedPacket::from_bare(packet, STEEL_CONFIG.compression, ConnectionProtocol::Play)
    {
        player.connection.send_encoded(encoded);
    }
}