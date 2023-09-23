use crate::{device_error_or, Device};

use anyhow::Result;

use super::Geometry;

pub struct UserGeometry {
    handle: embree4_sys::RTCGeometry,
}

type ExternBoundsFn = unsafe extern "C" fn(args: *const embree4_sys::RTCBoundsFunctionArguments);
type ExternIntersectFn =
    unsafe extern "C" fn(args: *const embree4_sys::RTCIntersectFunctionNArguments);
type ExternOccludedFn =
    unsafe extern "C" fn(_args: *const embree4_sys::RTCOccludedFunctionNArguments);

#[allow(clippy::missing_safety_doc)]
impl UserGeometry {
    /// Creates a new `UserGeometry` object.
    ///
    /// # Arguments
    ///
    /// * `device` - The Embree device.
    /// * `data` - The user-defined data associated with the geometry.
    /// * `bounds_fn` - The function pointer to the bounds function.
    /// * `intersect_fn` - The function pointer to the intersect function.
    /// * `occluded_fn` - The function pointer to the occluded function.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `UserGeometry` object if successful, or an `anyhow::Error` if an error occurred.
    pub fn try_new<T>(
        device: &Device,
        data: &T,
        bounds_fn: ExternBoundsFn,
        intersect_fn: ExternIntersectFn,
        occluded_fn: ExternOccludedFn,
    ) -> Result<Self> {
        let geometry = unsafe {
            embree4_sys::rtcNewGeometry(device.handle, embree4_sys::RTCGeometryType::USER)
        };
        device_error_or(device, (), "Could not create user geometry")?;

        unsafe {
            embree4_sys::rtcSetGeometryUserPrimitiveCount(geometry, 1);
        }
        device_error_or(device, (), "Could not set user geometry primitive count")?;

        let data_ptr = data as *const _ as _;

        unsafe {
            embree4_sys::rtcSetGeometryUserData(geometry, data_ptr);
        }
        device_error_or(device, (), "Could not set user geometry data")?;

        unsafe {
            embree4_sys::rtcSetGeometryBoundsFunction(geometry, Some(bounds_fn), data_ptr);
        };
        device_error_or(device, (), "Could not set user geometry bounds function")?;

        unsafe {
            embree4_sys::rtcSetGeometryIntersectFunction(geometry, Some(intersect_fn));
        }
        device_error_or(device, (), "Could not set user geometry intersect function")?;

        unsafe {
            embree4_sys::rtcSetGeometryOccludedFunction(geometry, Some(occluded_fn));
        }
        device_error_or(device, (), "Could not set user geometry occluded function")?;

        unsafe {
            embree4_sys::rtcCommitGeometry(geometry);
        }
        device_error_or(device, (), "Could not commit user geometry")?;

        Ok(Self { handle: geometry })
    }
}

impl Geometry for UserGeometry {
    fn geometry(&self) -> embree4_sys::RTCGeometry {
        self.handle
    }
}

impl Drop for UserGeometry {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseGeometry(self.handle);
        }
    }
}
