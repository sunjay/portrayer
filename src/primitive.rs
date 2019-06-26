mod sphere;
mod triangle;

pub use sphere::*;

use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};

#[derive(Debug)]
pub enum Primitive {
    Sphere(Sphere),
}

impl From<Sphere> for Primitive {
    fn from(sphere: Sphere) -> Self {
        Primitive::Sphere(sphere)
    }
}

impl RayHit for Primitive {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        use Primitive::*;
        match self {
            Sphere(sphere) => sphere.ray_hit(ray, t_range),
        }
    }
}
