//! Clientbound Initialize World Border packet.
//!
//! Sent to initialize the world border for the client.
//!
//! Packet ID: 0x48 (Play)

use steel_macros::ClientPacket;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = 0x48)]
pub struct CInitializeBorder {
    /// X coordinate of the world border center.
    pub center_x: f64,
    /// Z coordinate of the world border center.
    pub center_z: f64,
    /// Current world border size (diameter).
    pub size: f64,
    /// Absolute maximum size of the world border.
    pub absolute_max_size: i32,
    /// Time in seconds until the warning appears (when using smooth warning).
    pub warning_time: i32,
    /// Distance at which the warning appears.
    pub warning_blocks: i32,
}

impl WriteTo for CInitializeBorder {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(&self.center_x.to_be_bytes())?;
        writer.write_all(&self.center_z.to_be_bytes())?;
        writer.write_all(&self.size.to_be_bytes())?;
        VarInt(self.absolute_max_size).write(writer)?;
        VarInt(self.warning_time).write(writer)?;
        VarInt(self.warning_blocks).write(writer)
    }
}

impl CInitializeBorder {
    /// Creates a new Initialize World Border packet.
    #[inline]
    pub fn new(
        center_x: f64,
        center_z: f64,
        size: f64,
        absolute_max_size: i32,
        warning_time: i32,
        warning_blocks: i32,
    ) -> Self {
        Self {
            center_x,
            center_z,
            size,
            absolute_max_size,
            warning_time,
            warning_blocks,
        }
    }

    /// Creates a default world border centered at the given position.
    #[inline]
    pub fn default_border(center_x: f64, center_z: f64) -> Self {
        Self::new(
            center_x,
            center_z,
            59_999_984.0,
            59_999_984,
            15,
            5,
        )
    }
}