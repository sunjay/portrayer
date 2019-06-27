//! Test for hierarchical ray-tracers.

use std::error::Error;
use std::path::Path;

use portrayer::{
    scene::{Scene, SceneNode, Geometry},
    primitive::{Sphere, Mesh, Cube},
    material::Material,
    light::Light,
    render::Target,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mut scene = Scene::new(
        SceneNode::default()
            .rotated_x(Radians::from_degrees(23.0))
            .translated((6.0, -2.0, -15.0)),
        Rgb {r: 0.3, g: 0.3, b: 0.3},
    );

    let gold = scene.add_material(Material {
        diffuse: Rgb {r: 0.9, g: 0.8, b: 0.4},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.4},
        shininess: 25.0,
        reflectivity: 0.0,
    });
    let grass = scene.add_material(Material {
        diffuse: Rgb {r: 0.1, g: 0.7, b: 0.1},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        reflectivity: 0.0,
    });
    let blue = scene.add_material(Material {
        diffuse: Rgb {r: 0.7, g: 0.6, b: 1.0},
        specular: Rgb {r: 0.5, g: 0.4, b: 0.8},
        shininess: 25.0,
        reflectivity: 0.0,
    });

    let plane = &tobj::load_obj(Path::new("assets/plane.obj"))?.0[0].mesh;
    let dodeca = &tobj::load_obj(Path::new("assets/dodeca.obj"))?.0[0].mesh;

    // The arc
    let mut arc = scene.add_node(SceneNode::default()
        .translated((0.0, 0.0, -10.0))
        .rotated_y(Radians::from_degrees(60.0)));
    scene.root_mut().add_child(arc);

    let p1 = scene.add_node(SceneNode::from(Geometry::new(Cube, gold))
        .scaled((0.8, 4.0, 0.8))
        .translated((-2.0, 2.0, 0.0)));
    arc.add_child(p1);

    let p2 = scene.add_node(SceneNode::from(Geometry::new(Cube, gold))
        .scaled((0.8, 4.0, 0.8))
        .translated((2.0, 2.0, 0.0)));
    arc.add_child(p2);

    let s = scene.add_node(SceneNode::from(Geometry::new(Sphere, gold))
        .scaled((4.0, 0.6, 0.6))
        .translated((0.0, 4.0, 0.0)));
    arc.add_child(s);

    // The floor
    let floor = scene.add_node(SceneNode::from(Geometry::new(Mesh::from(plane), grass))
        .scaled(30.0));
    scene.root_mut().add_child(floor);

    // Central "sphere"
    let poly = scene.add_node(SceneNode::from(Geometry::new(Mesh::from(dodeca), blue))
        .translated((-2.0, 1.618034, 0.0)));
    scene.root_mut().add_child(poly);

    // l1
    scene.add_light(Light {
        position: Vec3 {x: 200.0, y: 200.0, z: 400.0},
        color: Rgb {r: 0.8, g: 0.8, b: 0.8},
        falloff: Default::default(),
    });
    // l2
    scene.add_light(Light {
        position: Vec3 {x: 0.0, y: 5.0, z: -20.0},
        color: Rgb {r: 0.4, g: 0.4, b: 0.8},
        falloff: Default::default(),
    });

    let cam = CameraSettings {
        eye: (0.0, 0.0, 0.0).into(),
        center: (0.0, 0.0, -1.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = RgbImage::new(256, 256);

    image.draw(&scene, cam,
        |_, y| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - y) + Rgb::blue() * y);

    Ok(image.save("hier.png")?)
}
