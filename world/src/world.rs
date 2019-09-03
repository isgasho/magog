use crate::animations::{Anim, AnimState, Animations};
use crate::components;
use crate::event::Event;
use crate::flags::Flags;
use crate::fov::SightFov;
use crate::item::Slot;
use crate::location::{Location, Portal};
use crate::mutate::Mutate;
use crate::query::Query;
use crate::sector::{WorldSkeleton, SECTOR_WIDTH};
use crate::spatial::{Place, Spatial};
use crate::spec::EntitySpawn;
use crate::terraform::{Terraform, TerrainQuery};
use crate::terrain::Terrain;
use crate::volume::Volume;
use crate::world_cache::WorldCache;
use crate::Distribution;
use crate::Rng;
use calx::{seeded_rng, HexFov, HexFovIter};
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::slice;

pub const GAME_VERSION: &str = "0.1.0";

calx_ecs::build_ecs! {
    desc: components::Desc,
    anim: Anim,
    map_memory: components::MapMemory,
    health: components::Health,
    brain: components::Brain,
    item: components::Item,
    stats: components::StatsComponent,
    status: components::Statuses,
}

/// Toplevel game state object.
#[derive(Serialize, Deserialize)]
pub struct World {
    /// Game version. Not mutable in the slightest, but the simplest way to
    /// get versioned save files is to just drop it here.
    version: String,
    /// Entity component system.
    ecs: Ecs,
    /// Static startup game world
    world_cache: WorldCache,
    /// Spawns from worldgen that have been generated in world.
    generated_spawns: HashSet<(Location, EntitySpawn)>,
    /// Spatial index for game entities.
    spatial: Spatial,
    /// Global gamestate flags.
    flags: Flags,
    /// Persistent random number generator.
    rng: Rng,
    /// Event queue
    pub(crate) events: Vec<Event>,
}

impl World {
    pub fn new(seed: u32, skeleton: WorldSkeleton) -> World {
        let mut ret = World {
            version: GAME_VERSION.to_string(),
            ecs: Ecs::new(),
            world_cache: WorldCache::new(seed, skeleton),
            generated_spawns: Default::default(),
            spatial: Spatial::new(),
            flags: Flags::new(),
            rng: seeded_rng(&seed),
            events: Vec::new(),
        };

        // TODO non-lexical borrow
        let player_entry = ret.world_cache.player_entrance();
        ret.spawn_player(player_entry);
        ret.generate_world_spawns();

        ret
    }

    pub fn events(&self) -> &Vec<Event> { &self.events }

    pub(crate) fn clear_events(&mut self) { self.events.clear() }

    fn generate_world_spawns(&mut self) {
        let mut spawns = self.world_cache.drain_spawns();
        spawns.retain(|s| !self.generated_spawns.contains(s));
        let seed = self.rng_seed();

        for (loc, s) in &spawns {
            // Create one-off RNG from just the spawn info, will always run the same for same info.
            let mut rng = calx::seeded_rng(&(seed, loc, s));
            // Construct loadout from the spawn info and generate it in world.
            self.spawn(&s.sample(&mut rng), *loc);
            self.generated_spawns.insert((*loc, s.clone()));
        }
    }
}

impl TerrainQuery for World {
    fn is_valid_location(&self, _loc: Location) -> bool {
        //Location::origin().v2_at(loc).map_or(false, ::on_screen)
        true
    }

    fn terrain(&self, loc: Location) -> Terrain {
        let mut t = self.world_cache.get_terrain(loc);

        if t == Terrain::Door && self.has_mobs(loc) {
            // Standing in the doorway opens the door.
            t = Terrain::OpenDoor;
        }

        t
    }

    fn portal(&self, loc: Location) -> Option<Location> { self.world_cache.get_portal(loc) }

    fn is_untouched(&self, _loc: Location) -> bool { unimplemented!() }
}

impl Query for World {
    fn location(&self, e: Entity) -> Option<Location> {
        match self.spatial.get(e) {
            Some(Place::At(loc)) => Some(loc),
            Some(Place::In(container, _)) => self.location(container),
            _ => None,
        }
    }

    fn player(&self) -> Option<Entity> {
        if let Some(p) = self.flags.player {
            if self.is_alive(p) {
                return Some(p);
            }
        }

        None
    }

    fn get_tick(&self) -> u64 { self.flags.tick }

    fn rng_seed(&self) -> u32 { self.world_cache.seed() }

    fn entities(&self) -> slice::Iter<'_, Entity> { self.ecs.iter() }

    fn entities_at(&self, loc: Location) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn entities_in(&self, parent: Entity) -> Vec<Entity> { self.spatial.entities_in(parent) }

    fn ecs(&self) -> &Ecs { &self.ecs }

    fn entity_equipped(&self, parent: Entity, slot: Slot) -> Option<Entity> {
        self.spatial.entity_equipped(parent, slot)
    }

    fn entity_contains(&self, parent: Entity, child: Entity) -> bool {
        self.spatial.contains(parent, child)
    }

    fn sphere_volume(&self, origin: Location, radius: u32) -> Volume {
        Volume::sphere(self, origin, radius)
    }
}

impl Mutate for World {
    fn next_tick(&mut self) {
        self.generate_world_spawns();
        self.tick_anims();

        self.ai_main();

        self.clean_dead();
        self.flags.tick += 1;
    }

    fn set_entity_location(&mut self, e: Entity, loc: Location) { self.spatial.insert_at(e, loc); }

    fn equip_item(&mut self, e: Entity, parent: Entity, slot: Slot) {
        self.spatial.equip(e, parent, slot);
        self.rebuild_stats(parent);
    }

    fn set_player(&mut self, player: Option<Entity>) { self.flags.player = player; }

    fn spawn(&mut self, loadout: &Loadout, loc: Location) -> Entity {
        let e = loadout.make(&mut self.ecs);
        self.place_entity(e, loc);
        e
    }

    fn kill_entity(&mut self, e: Entity) { self.spatial.remove(e); }

    fn remove_entity(&mut self, e: Entity) { self.ecs.remove(e); }

    fn do_fov(&mut self, e: Entity) {
        if !self.ecs.map_memory.contains(e) {
            return;
        }

        if let Some(origin) = self.location(e) {
            const DEFAULT_FOV_RANGE: u32 = 7;
            const OVERLAND_FOV_RANGE: u32 = SECTOR_WIDTH as u32;

            // Long-range sight while in overworld.
            let range = if origin.z == 0 {
                OVERLAND_FOV_RANGE
            } else {
                DEFAULT_FOV_RANGE
            };

            let fov: HashSet<Location> = HashSet::from_iter(
                HexFov::new(SightFov::new(self, range, origin))
                    .add_fake_isometric_acute_corners(|pos, a| {
                        self.terrain(a.origin + pos).is_wall()
                    })
                    .map(|(pos, a)| a.origin + pos),
            );

            let memory = &mut self.ecs.map_memory[e];
            memory.seen.clear();

            for &loc in &fov {
                memory.seen.insert(loc);
                // Don't spread map memory to other sectors we haven't "properly" seen yet.
                // The edges are still shown in visual FOV for display niceness.
                if loc.sector() == origin.sector() {
                    memory.remembered.insert(loc);
                }
            }
        }
    }

    fn push_event(&mut self, event: Event) { self.events.push(event); }

    fn rng(&mut self) -> &mut Rng { &mut self.rng }

    fn ecs_mut(&mut self) -> &mut Ecs { &mut self.ecs }

    fn spawn_fx(&mut self, loc: Location) -> Entity {
        let e = self.ecs.make();
        self.place_entity(e, loc);
        let t = self.anim_tick();
        let mut anim = Anim::default();
        // Give it some starting state that makes it get cleaned up by default.
        // Doesn't matter which fx state in particular, we just don't want the 'Mob' default state,
        // since that denotes an permanent entity.
        anim.state = AnimState::Explosion;
        self.ecs.anim.insert(e, anim);
        e
    }
}

impl Terraform for World {
    fn set_terrain(&mut self, _loc: Location, _terrain: Terrain) {
        unimplemented!();
    }

    fn set_portal(&mut self, _loc: Location, _portal: Portal) {
        unimplemented!();
    }

    fn remove_portal(&mut self, _loc: Location) {
        unimplemented!();
    }
}

impl Animations for World {
    fn anim_tick(&self) -> u64 { self.flags.anim_tick }
    fn anim(&self, e: Entity) -> Option<&Anim> { self.ecs.anim.get(e) }
    fn anim_mut(&mut self, e: Entity) -> Option<&mut Anim> { self.ecs.anim.get_mut(e) }

    fn tick_anims(&mut self) {
        let entities: Vec<Entity> = self.entities().cloned().collect();
        for e in entities {
            self.anim_mut(e).map(|a| a.tick());

            if self.is_expired_fx(e) {
                self.remove_entity(e);
            }
        }
        self.flags.anim_tick += 1;
    }
}
