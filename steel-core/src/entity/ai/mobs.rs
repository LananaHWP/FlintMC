//! Mob-specific AI setup for all vanilla mob types.
//!
//! This module applies the correct AI goals and targets based on entity type.

use steel_utils::Identifier;

use crate::entity::ai::goal::behaviors::*;
use crate::entity::ai::target::HurtByTargetGoal;
use crate::entity::ai::MobAI;
use crate::entity::ai::target::NearestAttackableTargetGoal;

pub fn apply_mob_ai(ai: &MobAI, entity_type_key: &Identifier) {
    let key_str = format!("{}:{}", entity_type_key.namespace, entity_type_key.path);
    let short_name: &str = key_str.rsplit(':').next().unwrap_or(&key_str);

    match short_name {
        "zombie" | "zombie_villager" | "zombie_villager_v2" => apply_zombie_ai(ai),
        "husk" => apply_husk_ai(ai),
        "drowned" => apply_drowned_ai(ai),
        "skeleton" | "stray" => apply_skeleton_ai(ai),
        "wither_skeleton" => apply_wither_skeleton_ai(ai),
        "creeper" => apply_creeper_ai(ai),
        "spider" => apply_spider_ai(ai),
        "cave_spider" => apply_cave_spider_ai(ai),
        "enderman" => apply_enderman_ai(ai),
        "slime" => apply_slime_ai(ai),
        "wither" => apply_wither_ai(ai),
        "blaze" => apply_blaze_ai(ai),
        "piglin" => apply_piglin_ai(ai),
        "piglin_brute" => apply_piglin_brute_ai(ai),
        "zombified_piglin" => apply_zombified_piglin_ai(ai),
        "hoglin" => apply_hoglin_ai(ai),
        "zoglin" => apply_zoglin_ai(ai),
        "shulker" => apply_shulker_ai(ai),
        "phantom" => apply_phantom_ai(ai),
        "vex" => apply_vex_ai(ai),
        "warden" => apply_warden_ai(ai),
        "cow" => apply_cow_ai(ai),
        "pig" => apply_pig_ai(ai),
        "sheep" => apply_sheep_ai(ai),
        "chicken" => apply_chicken_ai(ai),
        "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" => apply_horse_ai(ai),
        "wolf" => apply_wolf_ai(ai),
        "cat" | "tamed_cat" => apply_cat_ai(ai),
        "ocelot" => apply_ocelot_ai(ai),
        "rabbit" => apply_rabbit_ai(ai),
        "bat" => apply_bat_ai(ai),
        "parrot" => apply_parrot_ai(ai),
        "llama" | "trader_llama" => apply_llama_ai(ai),
        "polar_bear" => apply_polar_bear_ai(ai),
        "panda" => apply_panda_ai(ai),
        "fox" => apply_fox_ai(ai),
        "bee" => apply_bee_ai(ai),
        "iron_golem" | "village_golem" => apply_iron_golem_ai(ai),
        "snow_golem" => apply_snow_golem_ai(ai),
        "pillager" | "raider" => apply_pillager_ai(ai),
        "vindicator" => apply_vindicator_ai(ai),
        "illusioner" => apply_illusioner_ai(ai),
        "witch" => apply_witch_ai(ai),
        "dolphin" => apply_dolphin_ai(ai),
        "squid" | "glow_squid" => apply_squid_ai(ai),
        "axolotl" => apply_axolotl_ai(ai),
        "turtle" => apply_turtle_ai(ai),
        "cod" | "salmon" | "tropical_fish" | "pufferfish" => apply_fish_ai(ai),
        "elder_guardian" => apply_elder_guardian_ai(ai),
        "guardian" => apply_guardian_ai(ai),
        "magma_cube" => apply_magma_cube_ai(ai),
        "ender_dragon" => apply_ender_dragon_ai(ai),
        _ => apply_default_mob_ai(ai),
    }
}

fn apply_zombie_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(BreakDoorGoal::new()), 4);
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_husk_ai(ai: &MobAI) {
    apply_zombie_ai(ai);
}

fn apply_drowned_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(BreakDoorGoal::new()), 4);
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FloatGoal::new(0.04)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_skeleton_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RangedAttackGoal::new(20)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_wither_skeleton_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_creeper_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_spider_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(ClimbGoal::new()), 1);
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LeapAtTargetGoal::new(0.5, 0.4)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_cave_spider_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(LeapAtTargetGoal::new(0.7, 0.4)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 4);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_enderman_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TeleportWhenTargetGoneGoal::new()), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 3);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_slime_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_wither_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_blaze_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FloatGoal::new(0.04)), 6);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_piglin_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 2);
    goals.add_goal(Box::new(AvoidEntityGoal::new(6.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_piglin_brute_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_zombified_piglin_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_hoglin_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(AvoidEntityGoal::new(4.0)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 4);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_zoglin_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_shulker_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 1);
    drop(goals);
}

fn apply_phantom_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_vex_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_warden_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_cow_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);
}

fn apply_pig_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);
}

fn apply_sheep_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 6);
    drop(goals);
}

fn apply_chicken_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(FloatGoal::new(0.01)), 5);
    drop(goals);
}

fn apply_horse_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(JumpWithOwnerGoal::new()), 4);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 5);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 6);
    drop(goals);
}

fn apply_wolf_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(8.0)), 1);
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 2);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 3);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 4);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 5);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 6);
    goals.add_goal(Box::new(SitGoal::new()), 7);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_tamed_wolf_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(OwnerHurtByTargetGoal::new()), 1);
    goals.add_goal(Box::new(OwnerTargetGoal::new()), 1);
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 2);
    goals.add_goal(Box::new(SitGoal::new()), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_cat_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(SitGoal::new()), 4);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 5);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 6);
    drop(goals);
}

fn apply_ocelot_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(4.0)), 1);
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    drop(goals);
}

fn apply_rabbit_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(4.0)), 1);
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 2);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    drop(goals);
}

fn apply_bat_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 1);
    drop(goals);
}

fn apply_parrot_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    goals.add_goal(Box::new(FloatGoal::new(0.02)), 4);
    drop(goals);
}

fn apply_llama_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(6.0)), 1);
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 2);
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);
}

fn apply_polar_bear_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 2);
    goals.add_goal(Box::new(AvoidEntityGoal::new(8.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
}

fn apply_panda_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 5);
    drop(goals);
}

fn apply_fox_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(8.0)), 1);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    drop(goals);
}

fn apply_bee_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(FindEntityGoal::new(8.0)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    goals.add_goal(Box::new(FloatGoal::new(0.02)), 4);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 5);
    drop(goals);
}

fn apply_iron_golem_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(FindEntityGoal::new(10.0)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 4);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 5);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_snow_golem_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(TemptGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    goals.add_goal(Box::new(RandomLookaroundGoal::new()), 4);
    goals.add_goal(Box::new(AvoidEntityGoal::new(4.0)), 5);
    drop(goals);
}

fn apply_pillager_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RangedAttackGoal::new(20)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_vindicator_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_illusioner_ai(ai: &MobAI) {
    apply_vindicator_ai(ai);
}

fn apply_witch_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RangedAttackGoal::new(20)), 1);
    goals.add_goal(Box::new(AvoidEntityGoal::new(8.0)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 4);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(HurtByTargetGoal::new()), 1);
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 2);
}

fn apply_dolphin_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(FindEntityGoal::new(10.0)), 2);
    goals.add_goal(Box::new(FloatGoal::new(0.01)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    drop(goals);
}

fn apply_squid_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(6.0)), 1);
    goals.add_goal(Box::new(FloatGoal::new(0.04)), 2);
    goals.add_goal(Box::new(MoveBackToYGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    drop(goals);
}

fn apply_axolotl_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(AvoidEntityGoal::new(4.0)), 2);
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(FloatGoal::new(0.01)), 5);
    goals.add_goal(Box::new(MoveBackToYGoal::new(1.0)), 6);
    drop(goals);
}

fn apply_turtle_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(BreedGoal::new(1.0)), 1);
    goals.add_goal(Box::new(FollowParentGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FloatGoal::new(0.04)), 3);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 4);
    goals.add_goal(Box::new(MoveBackToYGoal::new(1.0)), 5);
    drop(goals);
}

fn apply_fish_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(AvoidEntityGoal::new(6.0)), 1);
    goals.add_goal(Box::new(FloatGoal::new(0.04)), 2);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 3);
    goals.add_goal(Box::new(MoveBackToYGoal::new(1.0)), 4);
    drop(goals);
}

fn apply_elder_guardian_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RangedAttackGoal::new(20)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FloatGoal::new(0.02)), 3);
    drop(goals);
}

fn apply_guardian_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(FloatGoal::new(0.04)), 3);
    drop(goals);
}

fn apply_magma_cube_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

#[allow(dead_code)]
fn apply_fireball_ai(_ai: &MobAI) {}

#[allow(dead_code)]
fn apply_wither_skull_ai(_ai: &MobAI) {}

#[allow(dead_code)]
fn apply_llama_spit_ai(_ai: &MobAI) {}

#[allow(dead_code)]
fn apply_shulker_bullet_ai(_ai: &MobAI) {}

#[allow(dead_code)]
fn apply_armor_stand_ai(_ai: &MobAI) {}

#[allow(dead_code)]
fn apply_minecart_ai(_ai: &MobAI) {}

fn apply_ender_dragon_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(MeleeAttackGoal::new(1.0)), 1);
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 3);
    drop(goals);

    let mut targets = ai.target_selector().lock();
    targets.add_target(Box::new(NearestAttackableTargetGoal::new()), 1);
}

fn apply_default_mob_ai(ai: &MobAI) {
    let mut goals = ai.goal_selector().lock();
    goals.add_goal(Box::new(RandomStrollGoal::new(1.0)), 2);
    goals.add_goal(Box::new(LookAtPlayerGoal::new()), 5);
    drop(goals);
}