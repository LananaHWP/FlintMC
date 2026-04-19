[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=plastic&logo=rust&logoColor=white)](https://www.youtube.com/watch?v=cE0wfjsybIQ&t=73s)
[![License](https://img.shields.io/github/license/LananaHWP/FlintMC?style=social)](https://github.com/LananaHWP/FlintMC/blob/master/LICENSE)
[![FlintMC](https://dcbadge.limes.pink/api/server/MwChEHnAbh?style=social)](https://discord.gg/MwChEHnAbh)
[![DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/LananaHWP/FlintMC)
![Tests](https://github.com/LananaHWP/FlintMC/actions/workflows/test.yml/badge.svg)
![Lint](https://github.com/LananaHWP/FlintMC/actions/workflows/lint.yml/badge.svg)
![Build](https://github.com/LananaHWP/FlintMC/actions/workflows/release.yml/badge.svg)



<div align="center">

# FlintMC

![Logo](https://i.imgur.com/lFQ6jH2.png)

Flint is a Rust implementation of the Minecraft Java Edition 26.1 server.
It focuses on clean code, performance, extensibility, and full vanilla parity.

![Demo](https://github.com/user-attachments/assets/ee656153-0660-4626-8295-37d3c96d8fd9)


</div>

---

## Working Features

### World Generation
- ✅ Noise-based terrain generation
- ✅ All biomes from vanilla datapacks
- ✅ Cave carvers with noise-based cave algorithm
- ✅ Biome decoration (8 stages)
- ✅ Ore generation (all ore types)
- ✅ Structure generation (villages, desert pyramids, jungle temples, swamp huts, ocean monuments, mineshafts, strongholds, nether fortresses, bastion remnants, ruined portals, shipwrecks, dungeons, igloos, end cities, ancient cities, woodland mansions, trial chambers, trail ruins)
- ✅ Jigsaw template pool loading (basic)
- ✅ Biome spawn rules

### Block Behaviors
- ✅ Redstone system (wire, torches, buttons, levers, repeaters, comparators)
- ✅ Container blocks (chest, barrel, hopper, furnace, brewing stand)
- ✅ Portal blocks (nether portal, end portal frame)
- ✅ Farming (farmland, crops, cactus, sugar cane)
- ✅ Fluids (water, lava flow)
- ✅ Functional blocks (dispenser, dropper, crafter, crafting table, stonecutter, loom, grindstone, bell, jukebox, anvil, beacon, note block, smithing table)
- ✅ Signs (floor/wall/hanging)
- ✅ 17+ additional block behaviors implemented

### Entity System
- ✅ MobEntity with full AI goal system
- ✅ AI Goals: RandomStroll, Float, MeleeAttack, Tempt, Climb, TeleportWhenTargetGone, BreakDoor, LookAtPlayer, and 20+ more
- ✅ Mob-specific AI (zombie, skeleton, creeper, spider, enderman, slime, blaze, piglin, cow, pig, sheep, chicken, horse, wolf, cat, rabbit, dolphin, squid, iron golem, snow golem)
- ✅ Light level checks for mob spawning
- ✅ Player proximity checks
- ✅ Biome-specific spawn restrictions

### Inventory/Menus
- ✅ Player inventory, crafting table, chest menus
- ✅ FurnaceMenu, BrewingStandMenu, HopperMenu, AnvilMenu, BeaconMenu, SmithingMenu
- ✅ Full slot system with permissions
- ✅ Recipe system

### Commands
- ✅ msg/tell/w commands
- ✅ scoreboard commands
- ✅ team commands
- ✅ Basic command dispatcher

### World Mechanics
- ✅ World border (CInitializeBorder packet)
- ✅ Spawn protection zone
- ✅ Weather system (rain/thunder)
- ✅ Time/daylight cycle
- ✅ Random block ticks
- ✅ Scheduled block ticks

### Redstone
- ✅ Redstone tick system with phases
- ✅ Signal propagation
- ✅ Repeater delay
- ✅ Comparator (compare/subtract modes)
- ✅ Button unlatch
- ✅ Torch burnout

### Lighting
- ✅ Block light propagation
- ✅ Transparent block handling
- ✅ Light emission calculation

### Tags & Registry
- ✅ Full block/item/entity tags
- ✅ Damage types
- ✅ Attributes
- ✅ Biome registry

### Networking
- ✅ Full packet protocol (Play state)
- ✅ Chunk synchronization
- ✅ Entity tracking

---

## TODO (Not Yet Implemented)

### High Priority
- [ ] Potion Effects system (framework exists, needs full implementation)
- [ ] Fire spread mechanics
- [ ] Enchantment application
- [ ] Projectiles (arrows, fireballs, tridents)
- [ ] Advancements system
- [ ] Mob breeding

### Medium Priority
- [ ] Stonecutter recipes
- [ ] Smithing recipes (trim/transform)
- [ ] Furnace/smoking/blasting recipes
- [ ] Campfire cooking
- [ ] Full ambient sounds
- [ ] Particle type registry
- [ ] Leaf decay

### Minor Priority
- [ ] Mycelium/grass spread
- [ ] Ice sliding physics
- [ ] Scaffolding behavior
- [ ] Observer block
- [ ] Conduit power
- [ ] TNT explosives
- [ ] Mob leashing

---

## 🔗 Links
<div align="center">

[Discord](https://discord.gg/MwChEHnAbh) | [GitCraft](https://github.com/WinPlay02/GitCraft) | [Fork from SteelMC](https://github.com/4lve/SteelMC)
</div>

---

## ⚙ How to Contribute

1. Identify a feature you'd like to add or an issue to work on.
   You should always create a post in the channel [feature-discussion](https://canary.discord.com/channels/1428487339759370322/1429074039015473272) when considering adding a major feature.
2. Decompile Minecraft 26.1 by running the provided script:
   ```bash
   ./update-minecraft-src.sh
   ```
   This will clone GitCraft and generate the decompiled source in `minecraft-src/`.
3. Fork the `master` branch of this repository.
4. Examine the vanilla implementation and translate it into idiomatic Rust as cleanly and efficiently as possible.
5. Commit your changes to your fork and open a pull request.

> [!NOTE]
> It is highly recommended to join the [Discord server](https://discord.gg/MwChEHnAbh) and reach out to [4lve](https://github.com/4lve) if you have code-related questions or encounter any ambiguities.

> [!IMPORTANT]
> This project is still in a very early stage of development.

### Precommit Hook
This repository uses [prek](https://prek.j178.dev/) to ensure that all commits follow the style guide and makes sure the cicd will pass.
To install the hook, some things needed to be installed first:
```bash
cargo install prek typos-cli --locked
```

Then you can run `prek install` to install the hook and it is configured to run automatically before every commit.
It will fix some things already for you, but the commit will still fail and please check the changes.