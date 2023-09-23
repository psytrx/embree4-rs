mod tri_mesh;
mod user;

pub use tri_mesh::*;
pub use user::*;

/// A trait implemented by all geometry types.
/// If you want to implement your own geometry type, you must implement this trait.
///
/// Make sure to release the geometry handle when the geometry is dropped.
/// See [rtcReleaseGeometry](https://github.com/embree/embree/blob/master/doc/src/api/rtcReleaseGeometry.md).
pub trait Geometry {
    /// Returns the handle of the geometry.
    fn geometry(&self) -> embree4_sys::RTCGeometry;
}
