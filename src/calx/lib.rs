#[crate_id = "calx"];
#[desc = "Shared gamelib"];
#[license = "MIT"];
#[feature(globs)];
#[feature(macro_rules)];
#[crate_type = "rlib"];

extern crate collections;
extern crate cgmath;
extern crate color;

pub mod text;
pub mod pack_rect;
pub mod rectutil;
pub mod gen_id;
pub mod app;
pub mod renderer;
pub mod sprite;
pub mod key;
