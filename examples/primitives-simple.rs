//! A simple scene that demonstrates some of the extra supported primitives

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cylinder, Cone, Plane},
    material::Material,
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mat_grass = Arc::new(Material {
        diffuse: Rgb {r: 0.173224, g: 0.8, b: 0.226505},
        ..Material::default()
    });

    let mat_cylinder = Arc::new(Material {
        diffuse: Rgb {r: 0.139339, g: 0.435762, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_cone = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.047361, b: 0.04305},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Cylinder, mat_cylinder))
                .scaled(2.0)
                .translated((-2.0, 1.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cone, mat_cone))
                .scaled(2.0)
                .translated((2.0, 1.0, 0.0))
                .into(),

            // Floor
            SceneNode::from(Geometry::new(Plane, mat_grass))
                .scaled(10.0)
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 10.0, z: 9.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.760838, 8.095396, 10.50759).into(),
        center: (-0.41716, -3.477774, -5.761218).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = Image::new("primitives-simple.png", 910, 512)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}
