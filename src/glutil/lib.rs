#[crate_id = "glutil"];
#[desc = "OpenGL rendering utilities"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(macro_rules)];

extern mod opengles;
extern mod calx;
extern mod stb;
extern mod cgmath;
extern mod glfw = "glfw-rs";

#[macro_escape]
pub mod gl_check;

pub mod mesh;
pub mod shader;
pub mod texture;
pub mod fonter;
pub mod app;
pub mod atlas;