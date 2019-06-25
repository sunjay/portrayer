//! The vek crate that we use to provide math primitives is very generic, but we will always want
//! to use it with floats. This module exports type aliases that allow us to not have to specify
//! that we are using "f64" all the time.

pub type Vec2 = vek::Vec2<f64>;
pub type Vec3 = vek::Vec3<f64>;
pub type Vec4 = vek::Vec4<f64>;

pub type Mat2 = vek::Mat2<f64>;
pub type Mat3 = vek::Mat3<f64>;
pub type Mat4 = vek::Mat4<f64>;

pub type Aabb = vek::Aabb<f64>;

pub type Rgba = vek::Rgba<f64>;
pub type Rgb = vek::Rgb<f64>;

/// This is an "extension trait". It allows me to add methods to structs I did not define.
pub trait Vec3Ext {
    /// Interprets this as a point and applies the given transformation matrix
    fn transformed_point(self, trans: Mat4) -> Self;

    /// Interprets this as a direction and applies the given transformation matrix
    fn transformed_direction(self, trans: Mat4) -> Self;
}

impl Vec3Ext for Vec3 {
    fn transformed_point(self, trans: Mat4) -> Self {
        Vec3::from(trans * Vec4::from_point(self))
    }

    fn transformed_direction(self, trans: Mat4) -> Self {
        Vec3::from(trans * Vec4::from_direction(self))
    }
}

/// A "newtype" to represent a value with the unit "radians"
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Radians(f64);

impl Radians {
    pub fn from_degrees(value: f64) -> Self {
        Radians(value.to_radians())
    }

    pub fn from_radians(value: f64) -> Self {
        Radians(value)
    }

    /// Returns the actual value stored in this struct (in radians)
    pub fn get(self) -> f64 {
        self.0
    }
}
