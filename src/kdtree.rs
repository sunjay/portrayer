//! This implementation of a k-d tree is based on this technical report:
//!
//! > Donald S. Fussell and K. R. Subramanian. Fast Ray Tracing Using K-d Trees. Tech. rep.
//! > Austin, TX, USA: University of Texas at Austin, 1988.

mod kdscene;
mod kdmesh;
mod leaf;
mod node;

#[cfg(feature = "kdtree")]
pub(crate) use kdscene::*;
pub use kdmesh::*;
pub(crate) use leaf::*;
pub(crate) use node::*;
