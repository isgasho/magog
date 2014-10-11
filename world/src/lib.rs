#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Display independent world logic for Magog"]

extern crate num;
extern crate rand;
extern crate serialize;
extern crate calx;

pub use entity::{Entity};
pub use geom::{HexGeom, DIR6, DIR8};
pub use location::{Location, Chart};
pub use world::{init_world, load, save};

pub mod terrain;
pub mod mob;

mod area;
mod comp;
mod entity;
mod ecs;
mod egg;
mod fov;
mod geom;
mod geomorph;
mod geomorph_data;
mod location;
mod mapgen;
mod spatial;
mod world;

#[deriving(Eq, PartialEq, Show)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// General type of a game entity.
#[deriving(Eq, PartialEq, Show, Encodable, Decodable)]
pub enum EntityKind {
    /// An active, mobile entity like the player or the NPCs.
    MobKind(mob::MobType),
    /// An entity that can be picked up and used in some way.
    ItemKind, // TODO ItemType data.
    /// A background item that doesn't do much.
    PropKind,
    /// A static object that does things when stepped on.
    NodeKind,
}

#[deriving(PartialEq)]
pub enum Biome {
    Overland = 0b1,
    Dungeon  = 0b10,

    // For things showing up at a biome.
    Anywhere = 0b11111111,
}

pub struct AreaSpec {
    pub biome: Biome,
    pub depth: int,
}

impl AreaSpec {
    pub fn new(biome: Biome, depth: int) -> AreaSpec {
        AreaSpec { biome: biome, depth: depth }
    }

    pub fn can_hatch(&self, environment: &AreaSpec) -> bool {
        self.depth >= 0 && self.depth <= environment.depth &&
        (self.biome as int & environment.biome as int) != 0
    }
}
