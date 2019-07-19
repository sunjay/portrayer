//! Demonstrates some of the extra supported primitives

use std::error::Error;
use std::sync::Arc;
use std::iter::once;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Cylinder, Cone, Sphere, Mesh, MeshData, Shading, Plane},
    material::Material,
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mat_grass = Arc::new(Material {
        diffuse: Rgb {r: 0.177353, g: 0.334328, b: 0.169638},
        ..Material::default()
    });

    let castle = make_castle()?
        .translated((0.0, 0.0, -1.6))
        .into();

    let trees = make_trees()
        .into();

    let scene = HierScene {
        root: SceneNode::from(vec![
            castle,

            trees,

            // Floor
            SceneNode::from(Geometry::new(Plane, mat_grass))
                .scaled(30.0)
                .into()
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
        eye: (0.0, 4.311144, 17.370693).into(),
        center: (0.0, 2.133119, -7.534255).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = Image::new("primitives.png", 910, 512)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}

fn make_castle() -> Result<SceneNode, Box<dyn Error>> {
    let mat_dome = Arc::new(Material {
        diffuse: Rgb {r: 0.609065, g: 0.731162, b: 0.8},
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 1000.0,
        reflectivity: 0.3,
        ..Material::default()
    });
    let mat_castle = Arc::new(Material {
        diffuse: Rgb {r: 0.769051, g: 0.304112, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_castle_tower_top = Arc::new(Material {
        diffuse: Rgb {r: 0.352613, g: 0.42773, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_castle_door = Arc::new(Material {
        diffuse: Rgb {r: 0.176099, g: 0.115632, b: 0.054921},
        ..Material::default()
    });
    let mat_road = Arc::new(Material {
        diffuse: Rgb {r: 0.121484, g: 0.024035, b: 0.0},
        ..Material::default()
    });

    let mut nodes = Vec::new();

    let castle_width = 4.0;
    let castle_length = castle_width;
    let castle_height = 2.0;

    let dome_radius = castle_width / 2.0;

    let tower_height = castle_height * 1.5;
    let tower_width = 1.5;

    let tower_roof_height = 2.0;
    let tower_roof_width = tower_width + 0.1;

    // Main castle body
    nodes.push(
        SceneNode::from(Geometry::new(Cube, mat_castle.clone()))
            .scaled((castle_width, castle_height, castle_length))
            .translated((0.0, castle_height/2.0, 0.0))
            .into()
    );

    // Castle dome
    nodes.push(
        SceneNode::from(Geometry::new(Sphere, mat_dome.clone()))
            .scaled((dome_radius, castle_height, dome_radius))
            .translated((0.0, castle_height, 0.0))
            .into()
    );

    // Castle door
    let castle_door_model = Arc::new(MeshData::load_obj("assets/prim_castle_door.obj")?);
    nodes.push(
        SceneNode::from(Geometry::new(Mesh::new(castle_door_model, Shading::Smooth), mat_castle_door))
            .translated((0.0, 1.1, castle_length/2.0 + 0.1))
            .into()
    );

    // Road
    nodes.push(
        SceneNode::from(Geometry::new(Cube, mat_road.clone()))
            .scaled((2.0, 0.01, 4.0))
            .translated((0.0, 0.0, castle_length/2.0 + 2.0 - 0.3))
            .into()
    );

    // All 4 towers
    let tower = Arc::new(SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cylinder, mat_castle.clone()))
            .scaled((tower_width, tower_height, tower_width))
            .translated((0.0, tower_height / 2.0, 0.0))
            .into(),
        SceneNode::from(Geometry::new(Cone, mat_castle_tower_top.clone()))
            .scaled((tower_roof_width, tower_roof_height, tower_roof_width))
            .translated((0.0, tower_height + tower_roof_height / 2.0, 0.0))
            .into()
    ]));

    // Castle towers
    for &x in &[-1.0, 1.0] {
        for &z in &[-1.0, 1.0] {
            let tower_pos = Vec3 {
                x: castle_width / 2.0 * x,
                y: 0.0,
                z: castle_length / 2.0 * z,
            };

            nodes.push(
                SceneNode::from(tower.clone())
                    .translated(tower_pos)
                    .into()
            );
        }
    }

    Ok(SceneNode::from(nodes))
}

fn make_trees() -> SceneNode {
    let mat_tree_leaves = Arc::new(Material {
        diffuse: Rgb {r: 0.289596, g: 0.8, b: 0.308959},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_tree_trunk = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.441708, b: 0.115746},
        ..Material::default()
    });

    let tree = Arc::new(SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cylinder, mat_tree_trunk))
            .scaled((0.3, 2.0, 0.3))
            .translated((0.0, 1.0, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Cone, mat_tree_leaves))
            .scaled((1.0, 2.0, 1.0))
            .translated((0.0, 2.9, 0.0))
            .into(),
    ]));

    let tree_positions = &[
        // Trees to the right of the camera
        Vec3 {x: 4.225878, y: 0.0, z: 3.695781},
        Vec3 {x: 5.225877, y: 0.0, z: 2.895781},
        Vec3 {x: 4.125877, y: 0.0, z: 2.395781},
        Vec3 {x: 5.125877, y: 0.0, z: 1.595781},
        Vec3 {x: 6.525877, y: 0.0, z: 0.795781},
        Vec3 {x: 5.125877, y: 0.0, z: 0.395781},
        Vec3 {x: 5.925876, y: 0.0, z: -0.704219},
        Vec3 {x: 4.725877, y: 0.0, z: -1.30422},
        Vec3 {x: 3.425877, y: 0.0, z: -0.804219},
        Vec3 {x: 3.025877, y: 0.0, z: -2.204219},
        Vec3 {x: 4.225877, y: 0.0, z: -2.30422},
        Vec3 {x: 5.425877, y: 0.0, z: -2.50422},
        Vec3 {x: 6.525876, y: 0.0, z: -2.00422},
        Vec3 {x: 6.925876, y: 0.0, z: -3.50422},
        Vec3 {x: 5.825876, y: 0.0, z: -3.90422},
        Vec3 {x: 4.625876, y: 0.0, z: -3.70422},
        Vec3 {x: 3.425876, y: 0.0, z: -3.40422},
        Vec3 {x: 3.625876, y: 0.0, z: -4.80422},
        Vec3 {x: 5.025876, y: 0.0, z: -5.10422},
        Vec3 {x: 6.825876, y: 0.0, z: -5.00422},

        // Trees to the left of the camera
        Vec3 {x: -3.374122, y: 0.0, z: 3.79578},
        Vec3 {x: -4.874123, y: 0.0, z: 3.29578},
        Vec3 {x: -2.874123, y: 0.0, z: 2.39578},
        Vec3 {x: -4.374123, y: 0.0, z: 2.19578},
        Vec3 {x: -5.674122, y: 0.0, z: 1.79578},
        Vec3 {x: -5.974123, y: 0.0, z: 0.195781},
        Vec3 {x: -4.674122, y: 0.0, z: 0.395781},
        Vec3 {x: -3.574123, y: 0.0, z: 1.09578},
        Vec3 {x: -3.274122, y: 0.0, z: -0.204219},
        Vec3 {x: -4.674122, y: 0.0, z: -1.00422},
        Vec3 {x: -5.874123, y: 0.0, z: -1.20422},
        Vec3 {x: -5.874123, y: 0.0, z: -2.40422},
        Vec3 {x: -4.574122, y: 0.0, z: -2.40422},
        Vec3 {x: -3.474122, y: 0.0, z: -1.70422},
        Vec3 {x: -3.574123, y: 0.0, z: -3.30422},
        Vec3 {x: -5.374123, y: 0.0, z: -3.60422},
    ];

    let fallen_tree = SceneNode::from(tree.clone())
        .rotated_xzy((Radians::from_degrees(0.0), Radians::from_degrees(50.0), Radians::from_degrees(-80.0)))
        .translated((2.285154, 0.13965, 2.474418))
        .into();

    SceneNode::from(tree_positions.iter().map(|&tree_pos| {
        SceneNode::from(tree.clone())
            .translated(tree_pos)
            .into()
    }).chain(once(fallen_tree)).collect::<Vec<_>>())
}
