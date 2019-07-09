pub mod math;
pub mod ray;
pub mod light;
pub mod camera;
pub mod material;
pub mod primitive;
pub mod scene;
pub mod render;
pub mod texture;
pub mod reporter;

mod flat_scene;
mod bounding_box;
mod kdtree;

#[cfg(all(feature = "kdtree", feature = "flat_scene"))]
compile_error!("Please do not use the kdtree and flat_scene Cargo features together");
