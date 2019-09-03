use crate::location::Location;
use crate::query::Query;
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};

/// Trait for advancing animations.
///
/// Kept separate from trait `Mutate` to emphasize the contract that advancing animation logic must
/// never cause game logic changes.
pub trait Animations: Query + Sized {
    fn anim_tick(&self) -> u64;
    fn anim(&self, e: Entity) -> Option<&Anim>;
    fn anim_mut(&mut self, e: Entity) -> Option<&mut Anim>;

    /// Advance animations without ticking the world logic.
    ///
    /// Use this when waiting for player input to finish pending animations.
    fn tick_anims(&mut self);

    /// Return whether entity is a transient effect.
    fn is_fx(&self, e: Entity) -> bool {
        if let Some(anim) = self.anim(e) {
            use AnimState::*;
            match anim.state {
                Mob | MobHurt | MobBlocks => false,
                Explosion | Gib => true,
            }
        } else {
            false
        }
    }

    /// Return whether an entity is a transient effect and should be deleted.
    fn is_expired_fx(&self, e: Entity) -> bool {
        if let Some(anim) = self.anim(e) {
            // TODO: Common place to store exact durations of anims, clean the fx immediately when
            // its duration is over.
            // While waiting for that, just clean them up after around 10 seconds
            self.is_fx(e) && self.anim_tick() - anim.anim_start > 300
        } else {
            false
        }
    }

    /// If an entity is undergoing animation, return the current frame
    fn anim_frame(&self, e: Entity) -> Option<usize> {
        unimplemented!();
    }
}

/// Entity animation state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Anim {
    pub tween_from: Location,
    // TODO: Write tweening in terms of starting time anim tick, tween_start: u64
    pub tween_current: u32,
    pub tween_max: u32,

    /// Anim_tick when the animation started
    pub anim_start: u64,
    pub state: AnimState,
}

impl Anim {
    pub fn tick(&mut self) {
        if self.tween_current > 0 {
            self.tween_current -= 1;
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum AnimState {
    /// Mob decorator, doing nothing in particular
    Mob,
    /// Show mob hurt animation
    MobHurt,
    /// Show mob blocking autoexplore animation
    MobBlocks,
    /// An explosion
    Explosion,
    /// A death gib
    Gib,
}

impl Default for AnimState {
    fn default() -> Self { AnimState::Mob }
}
