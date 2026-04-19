use steel_utils::Identifier;

#[derive(Debug, Clone)]
pub struct Advancement {
    pub id: Identifier,
    pub parent: Option<Identifier>,
    pub display: Option<AdvancementDisplay>,
    pub criteria: Vec<Criterion>,
    pub requirements: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct AdvancementDisplay {
    pub icon: AdvancementIcon,
    pub title: Vec<u8>,
    pub description: Vec<u8>,
    pub frame_type: AdvancementFrame,
    pub background_texture: Option<Identifier>,
    pub show_toast: bool,
    pub announce_to_chat: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone)]
pub struct AdvancementIcon {
    pub item: Identifier,
    pub nbt: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementFrame {
    Task = 0,
    Goal = 1,
    Challenge = 2,
}

#[derive(Debug, Clone)]
pub struct Criterion {
    pub id: String,
    pub trigger: Identifier,
    pub conditions: Vec<u8>,
}

impl Advancement {
    pub fn new(id: Identifier) -> Self {
        Self {
            id,
            parent: None,
            display: None,
            criteria: Vec::new(),
            requirements: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: Identifier) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_display(mut self, display: AdvancementDisplay) -> Self {
        self.display = Some(display);
        self
    }

    pub fn with_criteria(mut self, criteria: Vec<Criterion>) -> Self {
        self.criteria = criteria;
        self
    }

    pub fn with_requirements(mut self, requirements: Vec<Vec<String>>) -> Self {
        self.requirements = requirements;
        self
    }
}