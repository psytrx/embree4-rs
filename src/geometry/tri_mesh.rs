use std::{mem::size_of, slice};

use anyhow::{bail, Result};

use crate::{device_error_or, Device};

use super::Geometry;

pub struct TriangleMeshGeometry {
    handle: embree4_sys::RTCGeometry,
}

impl TriangleMeshGeometry {
    /// Constructs a new `TriangleMeshGeometry` instance from the given vertices and indices.
    ///
    /// # Example
    /// ```
    /// use embree4_rs::{*, geometry::*};
    /// use embree4_sys::*;
    ///
    /// let vertices = [
    ///   (-1.0, -1.0, 0.0),
    ///   (1.0, -1.0, 0.0),
    ///   (1.0, 1.0, 0.0),
    ///   (-1.0, 1.0, 0.0),
    /// ];
    /// let indices = [(0, 1, 2), (2, 3, 0)];
    ///
    /// let device = Device::try_new(None).unwrap();
    /// let geometry = TriangleMeshGeometry::try_new(&device, &vertices, &indices).unwrap();
    /// let scene = Scene::try_new(&device, SceneOptions::default()).unwrap();
    /// scene.attach_geometry(&geometry);
    /// ```
    pub fn try_new(
        device: &Device,
        vertices: &[(f32, f32, f32)],
        indices: &[(u32, u32, u32)],
    ) -> Result<Self> {
        let geometry = unsafe {
            embree4_sys::rtcNewGeometry(device.handle, embree4_sys::RTCGeometryType::TRIANGLE)
        };
        if geometry.is_null() {
            bail!("Failed to create geometry: {:?}", device.error());
        }

        let vertex_buf_ptr = unsafe {
            embree4_sys::rtcSetNewGeometryBuffer(
                geometry,
                embree4_sys::RTCBufferType::VERTEX,
                0,
                embree4_sys::RTCFormat::FLOAT3,
                3 * size_of::<f32>(),
                vertices.len(),
            )
        };
        if vertex_buf_ptr.is_null() {
            bail!(
                "Failed to create triangle mesh vertex buffer: {:?}",
                device.error()
            );
        }
        device_error_or(device, (), "Failed not create triangle mesh vertex buffer")?;

        let vertex_buf =
            unsafe { slice::from_raw_parts_mut(vertex_buf_ptr as *mut f32, 3 * vertices.len()) };

        // copy vertices into buffer
        for (i, v) in vertices.iter().enumerate() {
            vertex_buf[3 * i] = v.0;
            vertex_buf[3 * i + 1] = v.1;
            vertex_buf[3 * i + 2] = v.2;
        }

        let index_buf_ptr = unsafe {
            embree4_sys::rtcSetNewGeometryBuffer(
                geometry,
                embree4_sys::RTCBufferType::INDEX,
                0,
                embree4_sys::RTCFormat::UINT3,
                3 * size_of::<u32>(),
                indices.len(),
            )
        };
        if index_buf_ptr.is_null() {
            bail!(
                "Failed to create triangle mesh index buffer: {:?}",
                device.error()
            );
        }
        device_error_or(device, (), "Failed to create triangle mesh index buffer")?;

        let index_buf =
            unsafe { slice::from_raw_parts_mut(index_buf_ptr as *mut u32, 3 * indices.len()) };

        // copy indices into buffer
        for (i, idx) in indices.iter().enumerate() {
            index_buf[3 * i] = idx.0;
            index_buf[3 * i + 1] = idx.1;
            index_buf[3 * i + 2] = idx.2;
        }

        unsafe {
            embree4_sys::rtcCommitGeometry(geometry);
        }
        device_error_or(device, (), "Failed to commit triangle mesh geometry")?;

        Ok(Self { handle: geometry })
    }
}

impl Drop for TriangleMeshGeometry {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseGeometry(self.handle);
        }
    }
}

impl Geometry for TriangleMeshGeometry {
    fn geometry(&self) -> embree4_sys::RTCGeometry {
        self.handle
    }
}
