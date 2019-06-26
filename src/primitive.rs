mod sphere;
mod triangle;
mod mesh;
mod plane;
mod cube;

pub use sphere::*;
pub use mesh::*;
pub use cube::*;

use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};

#[derive(Debug)]
pub enum Primitive {
    Sphere(Sphere),
    Mesh(Mesh),
    Cube(Cube),
}

impl From<Sphere> for Primitive {
    fn from(sphere: Sphere) -> Self {
        Primitive::Sphere(sphere)
    }
}

impl From<Mesh> for Primitive {
    fn from(mesh: Mesh) -> Self {
        Primitive::Mesh(mesh)
    }
}

impl From<Cube> for Primitive {
    fn from(cube: Cube) -> Self {
        Primitive::Cube(cube)
    }
}

impl RayHit for Primitive {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        use Primitive::*;
        match self {
            Sphere(sphere) => sphere.ray_hit(ray, t_range),
            Mesh(mesh) => mesh.ray_hit(ray, t_range),
            Cube(cube) => cube.ray_hit(ray, t_range),
        }
    }
}
