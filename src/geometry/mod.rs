mod tri_mesh;

pub use tri_mesh::*;

/// A trait implemented by all geometry types.
/// If you want to implement your own geometry type, you must implement this trait.
pub trait Geometry {
    /// Returns the handle of the geometry.
    fn geometry(&self) -> embree4_sys::RTCGeometry;
}
