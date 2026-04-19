use std::collections::HashMap;
use steel_utils::Identifier;

#[derive(Debug, Clone, Default)]
pub struct PlayerAdvancementData {
    pub advancements: HashMap<Identifier, PlayerAdvancementProgress>,
    pub criteria: HashMap<String, CriterionProgress>,
}

#[derive(Debug, Clone)]
pub struct PlayerAdvancementProgress {
    pub awarded: bool,
    pub criterion_progress: HashMap<String, CriterionProgress>,
}

#[derive(Debug, Clone, Default)]
pub struct CriterionProgress {
    pub obtained: bool,
    pub date: Option<i64>,
}

impl PlayerAdvancementData {
    pub fn new() -> Self {
        Self {
            advancements: HashMap::new(),
            criteria: HashMap::new(),
        }
    }

    pub fn track_criterion(&mut self, advancement_id: &Identifier, criterion_id: &str) -> bool {
        let progress = self.advancements.entry(advancement_id.clone()).or_default();
        
        let criterion_progress = progress.criterion_progress.entry(criterion_id.to_string()).or_default();
        
        if criterion_progress.obtained {
            return false;
        }
        
        criterion_progress.obtained = true;
        criterion_progress.date = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        );
        
        let key = format!("{}:{}", advancement_id, criterion_id);
        self.criteria.insert(key, criterion_progress.clone());
        
        true
    }

    pub fn has_criterion(&self, advancement_id: &Identifier, criterion_id: &str) -> bool {
        self.advancements
            .get(advancement_id)
            .and_then(|p| p.criterion_progress.get(criterion_id))
            .map(|p| p.obtained)
            .unwrap_or(false)
    }

    pub fn is_advancement_awarded(&self, advancement_id: &Identifier) -> bool {
        self.advancements
            .get(advancement_id)
            .map(|p| p.awarded)
            .unwrap_or(false)
    }
}

impl Default for PlayerAdvancementProgress {
    fn default() -> Self {
        Self {
            awarded: false,
            criterion_progress: HashMap::new(),
        }
    }
}