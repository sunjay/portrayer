//! Title: "The Computer Graphics Castle"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use rand::{Rng, SeedableRng, rngs::StdRng};

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Sphere, Cylinder, MeshData, Shading},
    kdtree::KDMesh,
    material::{Material, WATER_REFRACTION_INDEX},
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb, Uv},
};

fn main() -> Result<(), Box<dyn Error>> {
    let scene = HierScene {
        root: SceneNode::from(vec![
            castle()?
                .scaled(1.4)
                .translated((0.0, 0.0, -229.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 50.0, y: 110.0, z: -120.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 111.25914, 539.31665).into(),
        center: (0.0, -41.326401, -590.425537).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    // let mut image = Image::new("graphics-castle.png", 1920, 1080)?;
    let mut image = Image::new("graphics-castle.png", 533, 300)?;

    // image.slice_mut((152, 128), (382, 162)).render::<RenderProgress, _>(&scene, cam,
    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb {r: 0.086, g: 0.38, b: 0.745} * uv.v);

    Ok(image.save()?)
}

fn castle() -> Result<SceneNode, Box<dyn Error>> {
    let mat_castle_walls = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_stairs_side = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_puppet = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_grass = Arc::new(Material {
        diffuse: Rgb {r: 0.376, g: 0.502, b: 0.22},
        ..Material::default()
    });

    let castle_model = Arc::new(MeshData::load_obj("assets/castle.obj")?);
    let castle_stairs_side = Arc::new(MeshData::load_obj("assets/castle_stairs_side.obj")?);
    let castle_hill_model = Arc::new(MeshData::load_obj("assets/castle_hill.obj")?);
    let puppet_castle_left_tower_model = Arc::new(MeshData::load_obj("assets/puppet_castle_left_tower.obj")?);
    let puppet_castle_right_tower_model = Arc::new(MeshData::load_obj("assets/puppet_castle_right_tower.obj")?);

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(KDMesh::new(&castle_model, Shading::Flat), mat_castle_walls.clone()))
            .into(),

        SceneNode::from(Geometry::new(KDMesh::new(&castle_stairs_side, Shading::Flat), mat_stairs_side.clone()))
            .translated((-11.0, 5.0, 19.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&castle_stairs_side, Shading::Flat), mat_stairs_side.clone()))
            .translated((11.0, 5.0, 19.0))
            .into(),

        SceneNode::from(Geometry::new(KDMesh::new(&castle_hill_model, Shading::Smooth), mat_grass.clone()))
            .translated((0.0, 3.75, -15.75))
            .into(),

        SceneNode::from(Geometry::new(KDMesh::new(&puppet_castle_left_tower_model, Shading::Smooth), mat_puppet.clone()))
            .translated((30.0, 33.6, 19.0))
            .into(),
        SceneNode::from(Geometry::new(Cylinder, mat_castle_walls.clone()))
            .scaled(10.0)
            .translated((30.0, 5.0, 20.0))
            .into(),
        SceneNode::from(Geometry::new(KDMesh::new(&puppet_castle_right_tower_model, Shading::Smooth), mat_puppet.clone()))
            .translated((-30.0, 33.6, 19.0))
            .into(),
        SceneNode::from(Geometry::new(Cylinder, mat_castle_walls.clone()))
            .scaled(10.0)
            .translated((-30.0, 5.0, 20.0))
            .into(),
    ]))
}
