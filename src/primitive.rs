use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};

#[derive(Debug)]
pub enum Primitive {
}

impl RayHit for Primitive {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        unimplemented!()
    }
}
