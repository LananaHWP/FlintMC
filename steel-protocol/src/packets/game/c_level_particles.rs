use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_LEVEL_PARTICLES;

#[derive(WriteTo, ClientPacket, Clone, Debug)]
#[packet_id(Play = C_LEVEL_PARTICLES)]
pub struct CLevelParticles {
    #[write(as = VarInt)]
    pub particle_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub speed: f32,
    #[write(as = VarInt)]
    pub count: i32,
    pub long_distance: bool,
    #[write(as = VarInt)]
    pub data: i32,
}

impl CLevelParticles {
    #[must_use]
    pub fn new(
        particle_id: i32,
        x: f64,
        y: f64,
        z: f64,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
        speed: f32,
        count: i32,
        long_distance: bool,
        data: i32,
    ) -> Self {
        Self {
            particle_id,
            x,
            y,
            z,
            offset_x,
            offset_y,
            offset_z,
            speed,
            count,
            long_distance,
            data,
        }
    }

    #[must_use]
    pub fn simple(particle_id: i32, x: f64, y: f64, z: f64, count: i32) -> Self {
        Self::new(
            particle_id, x, y, z, 0.0, 0.0, 0.0, 0.0, count, false, 0,
        )
    }
}