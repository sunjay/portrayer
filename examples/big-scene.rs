//! A very large (resource intensive) scene with some miscellaneous geometry.
//!
//! Used to test k-d tree acceleration mechanism.

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Primitive, Sphere, Cube, Cone, Cylinder},
    material::Material,
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;
use rand::{
    Rng,
    SeedableRng,
    rngs::StdRng,
    seq::SliceRandom,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Want the result to be random but also completely reproducible
    let mut rng = StdRng::seed_from_u64(1234939301);

    let materials: Vec<_> = (0..15).map(|_| Arc::new(Material {
        diffuse: Rgb {r: rng.gen(), g: rng.gen(), b: rng.gen()},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    })).collect();

    let primitives: &[Primitive] = &[
        Sphere.into(),
        Cube.into(),
        Cone.into(),
        Cylinder.into(),
    ];

    let width = 800.0;
    let length = 800.0;
    let height = 800.0;

    let n = 10;

    let mut nodes = Vec::new();
    for i in 0..n {
        let x = i as f64 / (n - 1) as f64 * width - width / 2.0;
        for j in 0..n {
            let y = j as f64 / (n - 1) as f64 * length - length / 2.0;
            for k in 0..n {
                let z = k as f64 / (n - 1) as f64 * height - height / 2.0;

                let prim = primitives.choose(&mut rng).unwrap();
                let mat = materials.choose(&mut rng).unwrap();

                let scale_base = 30.0;
                let scale_increase = 30.0;

                let geo = Geometry::new(prim.clone(), mat.clone());
                let node = SceneNode::from(geo)
                .scaled(scale_increase * rng.gen::<f64>() + scale_base)
                .rotated_xzy(Radians::from_degrees(360.0 * rng.gen::<f64>()))
                .translated(Vec3 {x, y: y + rng.gen::<f64>() * 50.0, z});

                nodes.push(node.into());
            }
        }
    }

    let scene = HierScene {
        root: SceneNode::from(nodes).into(),
        lights: vec![
            // white_light
            Light {
                position: Vec3 {x: -100.0, y: 150.0, z: 400.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 100.0, y: -150.0, z: 800.0},
                color: Rgb {r: 0.7, g: 0.7, b: 0.7},
                ..Light::default()
            },
            // magenta_light
            Light {
                position: Vec3 {x: 400.0, y: 100.0, z: 150.0},
                color: Rgb {r: 0.7, g: 0.0, b: 0.7},
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 0.0, 1200.0).into(),
        center: (0.0, 0.0, 0.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = RgbImage::new(1980, 1020);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("big-scene.png")?)
}
