//! Demonstrates the smooth (Phong) shading feature

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Mesh, MeshData, Shading},
    material::Material,
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mat_rock = Arc::new(Material {
        diffuse: Rgb {r: 0.256361, g: 0.256361, b: 0.256361},
        specular: Rgb {r: 0.6, g: 0.6, b: 0.6},
        shininess: 50.0,
        ..Material::default()
    });
    let mat_cow = Arc::new(Material {
        diffuse: Rgb {r: 0.692066, g: 0.477245, b: 0.293336},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_monkey = Arc::new(Material {
        diffuse: Rgb {r: 0.261829, g: 0.8, b: 0.310477},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let monkey_mesh = Arc::new(MeshData::load_obj("assets/monkey.obj")?);
    let cow_mesh = Arc::new(MeshData::load_obj("assets/cow.obj")?);
    let flat_rock_mesh = Arc::new(MeshData::load_obj("assets/flat_rock.obj")?);
    let smooth_rock_mesh = Arc::new(MeshData::load_obj("assets/smooth_rock.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            // Flat objects

            SceneNode::from(Geometry::new(Mesh::new(monkey_mesh.clone(), Shading::Flat), mat_monkey.clone()))
                .rotated_y(Radians::from_degrees(45.0))
                .translated((-1.904434, 1.4, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(cow_mesh.clone(), Shading::Flat), mat_cow.clone()))
                .scaled(0.5)
                .rotated_y(Radians::from_degrees(-15.0))
                .translated((-4.2, 1.8, 4.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(flat_rock_mesh.clone(), Shading::Flat), mat_rock.clone()))
                .translated((-3.396987, -1.4, 2.286671))
                .into(),

            // Smooth objects

            SceneNode::from(Geometry::new(Mesh::new(monkey_mesh.clone(), Shading::Smooth), mat_monkey.clone()))
                .rotated_y(Radians::from_degrees(-45.0))
                .translated((1.242585, 1.4, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(cow_mesh.clone(), Shading::Smooth), mat_cow.clone()))
                .scaled(0.5)
                .rotated_y(Radians::from_degrees(205.0))
                .translated((3.8, 1.8, 4.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(smooth_rock_mesh.clone(), Shading::Smooth), mat_rock.clone()))
                .translated((3.271008, -1.406423, 2.372513))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 5.0, z: 10.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (1.062382, 0.54746, 22.827951).into(),
        center: (-0.813817, 0.424462, -8.112782).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(24.0),
    };

    let mut image = Image::new("smooth-shading.png", 910, 512)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}
