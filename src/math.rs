//! The vek crate that we use to provide math primitives is very generic, but we will always want
//! to use it with floats. This module exports type aliases that allow us to not have to specify
//! that we are using "f32" all the time.

pub type Vec2 = vek::Vec2<f32>;
pub type Vec3 = vek::Vec3<f32>;
pub type Vec4 = vek::Vec4<f32>;

pub type Mat2 = vek::Mat2<f32>;
pub type Mat3 = vek::Mat3<f32>;
pub type Mat4 = vek::Mat4<f32>;

pub type Aabb = vek::Aabb<f32>;
