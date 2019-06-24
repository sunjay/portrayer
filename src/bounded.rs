use crate::math::Aabb;
use crate::intersect::Intersect;

/// Axis-aligned bounds
pub trait AABounds {
    /// Returns the axis-aligned bounding box that represents the rough area of space that this
    /// object exists in
    fn axis_aligned_bounds(&self) -> Aabb;
}

/// Ray-tracing acceleration via a hierarchical bounding volume
pub struct BoundingVolume<T> {
    value: T,
    bounds: Aabb,
}

impl<T: AABounds> BoundingVolume<T> {
    pub fn new(value: T) -> Self {
        let bounds = value.axis_aligned_bounds();
        Self {
            value,
            bounds,
        }
    }
}

impl<T: Intersect> Intersect for BoundingVolume<T> {
    fn intersect(&self) -> Option<()> {
        //TODO: Check if the bounding box intersects before passing on to the lower object
        if unimplemented!() {
            self.value.intersect()
        } else {
            None
        }
    }
}
