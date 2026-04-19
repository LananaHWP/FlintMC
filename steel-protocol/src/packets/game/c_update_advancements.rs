use steel_macros::ClientPacket;
use steel_registry::packets::play::C_UPDATE_ADVANCEMENTS;
use steel_utils::codec::VarInt;
use steel_utils::Identifier;
use steel_utils::serial::{PrefixedWrite, WriteTo};

#[derive(Debug, Clone, ClientPacket)]
#[packet_id(Play = C_UPDATE_ADVANCEMENTS)]
pub struct CUpdateAdvancements {
    pub action: AdvancementAction,
    pub advancements: Vec<CAdvancement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementAction {
    Initialize = 0,
    Update = 1,
    Remove = 2,
}

impl From<AdvancementAction> for VarInt {
    fn from(action: AdvancementAction) -> Self {
        VarInt(action as i32)
    }
}

impl steel_utils::serial::WriteTo for CUpdateAdvancements {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt::from(self.action).write(writer)?;
        self.advancements.write_prefixed::<VarInt>(writer)
    }
}

#[derive(Debug, Clone)]
pub struct CAdvancement {
    pub id: Identifier,
    pub parent: Option<Identifier>,
    pub display: Option<CAdvancementDisplay>,
    pub criteria: Vec<CAdvancementCriterion>,
    pub requirements: Vec<Vec<String>>,
}

impl WriteTo for CAdvancement {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.id.write(writer)?;
        match &self.parent {
            Some(parent) => {
                1u8.write(writer)?;
                parent.write(writer)?;
            }
            None => {
                0u8.write(writer)?;
            }
        }
        match &self.display {
            Some(display) => {
                1u8.write(writer)?;
                display.write(writer)?;
            }
            None => {
                0u8.write(writer)?;
            }
        }
        self.criteria.write_prefixed::<VarInt>(writer)?;
        VarInt(self.requirements.len() as i32).write(writer)?;
        for req in &self.requirements {
            VarInt(req.len() as i32).write(writer)?;
            for r in req {
                r.write(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CAdvancementDisplay {
    pub icon: CAdvancementIcon,
    pub title: Vec<u8>,
    pub description: Vec<u8>,
    pub frame_type: AdvancementFrame,
    pub background_texture: Option<Identifier>,
    pub show_toast: bool,
    pub announce_to_chat: bool,
    pub hidden: bool,
}

impl WriteTo for CAdvancementDisplay {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.icon.write(writer)?;
        self.title.write_prefixed::<VarInt>(writer)?;
        self.description.write_prefixed::<VarInt>(writer)?;
        VarInt::from(self.frame_type as i32).write(writer)?;
        match &self.background_texture {
            Some(tex) => {
                1u8.write(writer)?;
                tex.write(writer)?;
            }
            None => {
                0u8.write(writer)?;
            }
        }
        self.show_toast.write(writer)?;
        self.announce_to_chat.write(writer)?;
        self.hidden.write(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementFrame {
    Task = 0,
    Goal = 1,
    Challenge = 2,
}

#[derive(Debug, Clone)]
pub struct CAdvancementIcon {
    pub item: Identifier,
    pub nbt: Option<Vec<u8>>,
}

impl WriteTo for CAdvancementIcon {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.item.write(writer)?;
        match &self.nbt {
            Some(nbt) => {
                1u8.write(writer)?;
                nbt.write_prefixed::<VarInt>(writer)?;
            }
            None => {
                0u8.write(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CAdvancementCriterion {
    pub id: String,
    pub trigger_id: Identifier,
    pub conditions: Vec<u8>,
}

impl WriteTo for CAdvancementCriterion {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.id.write(writer)?;
        self.trigger_id.write(writer)?;
        self.conditions.write_prefixed::<VarInt>(writer)?;
        Ok(())
    }
}