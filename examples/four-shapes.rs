//! A simple scene with four shapes on a white background

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Sphere, Cone, Cylinder},
    material::Material,
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mat_glass_base = Material {
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 100.0,
        ..Material::default()
    };

    let mat_sphere = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.0, b: 0.0},
        ..mat_glass_base.clone()
    });

    let mat_cube = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.158481, b: 0.8},
        ..mat_glass_base.clone()
    });

    let mat_cone = Arc::new(Material {
        diffuse: Rgb {r: 0.064785, g: 0.8, b: 0.174433},
        ..mat_glass_base.clone()
    });

    let mat_cylinder = Arc::new(Material {
        diffuse: Rgb {r: 0.127564, g: 0.016029, b: 0.8},
        ..mat_glass_base.clone()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Sphere, mat_sphere.clone()))
                .translated((-4.0, 0.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_cube.clone()))
                .scaled(1.6)
                .rotated_y(Radians::from_degrees(-17.5411))
                .translated((-1.1, 0.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cone, mat_cone.clone()))
                .scaled(1.8)
                .translated((1.5, 0.2, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cylinder, mat_cylinder.clone()))
                .scaled(1.6)
                .translated((4.0, 0.0, 0.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 3.0, z: 11.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.1, g: 0.1, b: 0.1},
    };

    let cam = CameraSettings {
        eye: (0.0, 6.473007, 15.607252).into(),
        center: (0.0, -2.181935, -5.702181).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(10.0),
    };

    let mut image = Image::new("four-shapes.png", 1920, 512)?;

    image.render::<RenderProgress, _>(&scene, cam, |_| Rgb::white());

    Ok(image.save()?)
}
