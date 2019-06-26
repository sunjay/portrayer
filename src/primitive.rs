mod sphere;
mod triangle;
mod mesh;
mod plane;

pub use sphere::*;
pub use mesh::*;

use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};

#[derive(Debug)]
pub enum Primitive {
    Sphere(Sphere),
    Mesh(Mesh),
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

impl RayHit for Primitive {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        use Primitive::*;
        match self {
            Sphere(sphere) => sphere.ray_hit(ray, t_range),
            Mesh(mesh) => mesh.ray_hit(ray, t_range),
        }
    }
}
