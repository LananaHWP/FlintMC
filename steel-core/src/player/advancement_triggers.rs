use steel_utils::Identifier;

pub mod criteria_triggers {
    use steel_utils::Identifier;

    pub const INVENTORY_CHANGED: Identifier = Identifier::new_static("minecraft", "inventory_changed");
    pub const KILL_ENTITY: Identifier = Identifier::new_static("minecraft", "kill_entity");
    pub const PLAYER_KILLED_ENTITY: Identifier = Identifier::new_static("minecraft", "player_killed_entity");
    pub const ENTER_BIOME: Identifier = Identifier::new_static("minecraft", "enter_biome");
    pub const CONSUME_ITEM: Identifier = Identifier::new_static("minecraft", "consume_item");
    pub const EFFECT_CHANGED: Identifier = Identifier::new_static("minecraft", "effect_changed");
    pub const TICK: Identifier = Identifier::new_static("minecraft", "tick");
    pub const LOCATION: Identifier = Identifier::new_static("minecraft", "location");
    pub const SPEED: Identifier = Identifier::new_static("minecraft", "speed");
    pub const FALL_ONE_BLOCK: Identifier = Identifier::new_static("minecraft", "fall_one_block");
    pub const JUMP: Identifier = Identifier::new_static("minecraft", "jump");
    pub const START_RIDING: Identifier = Identifier::new_static("minecraft", "start_riding");
    pub const EXIT_VEHICLE: Identifier = Identifier::new_static("minecraft", "exit_vehicle");
    pub const BRED: Identifier = Identifier::new_static("minecraft", "bred");
    pub const TAME_ANIMAL: Identifier = Identifier::new_static("minecraft", "tame_animal");
    pub const MOB_GRIEFING: Identifier = Identifier::new_static("minecraft", "mob_griefing");
    pub const GENERATE_CONTAINER_MAX: Identifier = Identifier::new_static("minecraft", "generate_container_max");
    pub const GENERATE_CONTAINER: Identifier = Identifier::new_static("minecraft", "generate_container");
    pub const BEEHIVE_INSPECT: Identifier = Identifier::new_static("minecraft", "beehive_inspect");
    pub const BOTTLES_EMPTY: Identifier = Identifier::new_static("minecraft", "bottles_empty");
    pub const FIREWORK_USE: Identifier = Identifier::new_static("minecraft", "firework_use");
    pub const CAST_ANGLER: Identifier = Identifier::new_static("minecraft", "cast_angler");
    pub const CATCH_FISH: Identifier = Identifier::new_static("minecraft", "catch_fish");
    pub const BALANCE_FLIGHT: Identifier = Identifier::new_static("minecraft", "balance_flight");
    pub const SLEEP_IN_BED: Identifier = Identifier::new_static("minecraft", "sleep_in_bed");
    pub const CLOSE_CURE_SKELETON_TRAP: Identifier = Identifier::new_static("minecraft", "close_cure_skeleton_trap");
    pub const CURE_SKELETON_TRAP: Identifier = Identifier::new_static("minecraft", "cure_skeleton_trap");
    pub const AVOID_VEX: Identifier = Identifier::new_static("minecraft", "avoid_vex");
    pub const ARMOR_CHANGE: Identifier = Identifier::new_static("minecraft", "armor_change");
    pub const ENCHANTED_ITEM: Identifier = Identifier::new_static("minecraft", "enchanted_item");
    pub const VOLUNTARY_EXPLORE: Identifier = Identifier::new_static("minecraft", "voluntary_explore");
    pub const WALK_ON_EMERALD_BLOCK: Identifier = Identifier::new_static("minecraft", "walk_on_emerald_block");
}