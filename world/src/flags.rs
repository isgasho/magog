use crate::location::Location;
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Flags {
    pub camera: Location,
    pub tick: u64,
    pub anim_tick: u64,
    pub player_acted: bool,
    /// Store the player entity here for fast access.
    pub player: Option<Entity>,
    pub depth: i32,
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            camera: Location::new(0, 0, 0),
            tick: 0,
            anim_tick: 0,
            player_acted: false,
            player: None,
            depth: 0,
        }
    }
}
