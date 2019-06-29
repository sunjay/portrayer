// A simple test scene featuring some spherical cows grazing around Stonehenge.
// "Assume that cows are spheres..."

use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use portrayer::{
    scene::{Scene, SceneNode, Geometry},
    primitive::{Sphere, Mesh, Cube},
    material::Material,
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let stone = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.7, b: 0.7},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        reflectivity: 0.0,
    });
    let grass = Arc::new(Material {
        diffuse: Rgb {r: 0.1, g: 0.7, b: 0.1},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        reflectivity: 0.0,
    });
    let cow_hide = Arc::new(Material {
        diffuse: Rgb {r: 0.84, g: 0.6, b: 0.53},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 20.0,
        reflectivity: 0.0,
    });

    let plane = &tobj::load_obj(Path::new("assets/plane.obj"))?.0[0].mesh;
    let buckyball = &tobj::load_obj(Path::new("assets/buckyball.obj"))?.0[0].mesh;

    // The arch
    let arc = Arc::new(SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cube, stone.clone()))
            .translated((-1.9, 0.5, 0.1))
            .scaled((0.8, 4.0, 0.8))
            .into(),

        SceneNode::from(Geometry::new(Cube, stone.clone()))
            .translated((2.1, 0.5, 0.1))
            .scaled((0.8, 4.0, 0.8))
            .into(),

        SceneNode::from(Geometry::new(Sphere, stone.clone()))
            .scaled((4.0, 0.6, 0.6))
            .translated((0.0, 4.0, 0.0))
            .into(),
    ]).translated((0.0, 0.0, -10.0)));

    // Instancing the arc
    let mut nodes: Vec<Arc<_>> = (1..=6).map(|i| {
        SceneNode::from(arc.clone())
            .rotated_y(Radians::from_degrees(60.0 * (i-1) as f64))
            .into()
    }).collect();

    // Let's assume that cows are spheres
    let cow = Arc::new(SceneNode::from(vec![
        // body
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(1.0)
            .translated((0.0, 0.0, 0.0))
            .into(),
        // head
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(0.6)
            .translated((0.9, 0.3, 0.0))
            .into(),
        // tail
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(0.2)
            .translated((-0.94, 0.34, 0.0))
            .into(),
        // lfleg
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(0.3)
            .translated((0.7, -0.7, -0.7))
            .into(),
        // lrleg
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(0.3)
            .translated((-0.7, -0.7, -0.7))
            .into(),
        // rfleg
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(0.3)
            .translated((0.7, -0.7, 0.7))
            .into(),
        // rrleg
        SceneNode::from(Geometry::new(Sphere, cow_hide.clone()))
            .scaled(0.3)
            .translated((-0.7, -0.7, 0.7))
            .into(),
    ]));

    // Use instancing on the cow model to place some actual cows in the scene
    let cows = [
        (Vec3 {x: 1.0, y: 1.3, z: 14.0}, Radians::from_degrees(20.0)),
        (Vec3 {x: 5.0, y: 1.3, z: -11.0}, Radians::from_degrees(180.0)),
        (Vec3 {x: -5.5, y: 1.3, z: -3.0}, Radians::from_degrees(-60.0)),
    ];

    for &(cow_pos, cow_rot) in &cows {
        nodes.push(SceneNode::from(cow.clone())
            .scaled(1.4)
            .rotated_y(cow_rot)
            .translated(cow_pos)
            .into())
    }

    // The floor
    let floor = SceneNode::from(Geometry::new(Mesh::from(plane), grass.clone()))
        .scaled(30.0)
        .into();
    nodes.push(floor);

    // Construct a central altar in the shape of a buckyball.  The
    // buckyball at the centre of the real Stonehenge was destroyed
    // in the great fire of 733 AD.

    let altar = SceneNode::from(Geometry::new(Mesh::from(buckyball), stone.clone()))
        .scaled(1.5)
        .into();
    nodes.push(altar);

    let scene = Scene {
        root: SceneNode::from(nodes)
            .rotated_x(Radians::from_degrees(23.0))
            .into(),
        lights: vec![
            Light {
                position: Vec3 {x: 200.0, y: 202.0, z: 430.0},
                color: Rgb {r: 0.8, g: 0.8, b: 0.8},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.4, g: 0.4, b: 0.4},
    };

    let cam = CameraSettings {
        eye: (0.0, 2.0, 30.0).into(),
        center: (0.0, 2.0, 29.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = RgbImage::new(256, 256);

    image.render::<RenderProgress, _>(&scene, cam,
        |_, y| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - y) + Rgb::blue() * y);

    Ok(image.save("simple-cows.png")?)
}
