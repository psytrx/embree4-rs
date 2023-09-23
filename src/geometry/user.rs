use std::{marker::PhantomData, ptr};

use crate::{device_error_or, Device};

use anyhow::Result;
use embree4_sys::{RTCRayHit, RTC_INVALID_GEOMETRY_ID};

use super::Geometry;

/// The user geometry implementation.
/// If you want to use custom geometry, you need to implement this trait.
/// See the [examples/](https://github.com/psytrx/embree4-rs/tree/main/examples) for an example of
/// how to implement one.
pub trait UserGeometryImpl {
    /// Returns the bounds of the geometry
    fn bounds(&self) -> embree4_sys::RTCBounds;

    /// Computes an intersection between the given ray and the geometry.
    /// If an intersection is found,
    ///
    /// * the ray's `tfar` field
    /// * the hit's normals (`Ng_x`, `Ng_y`, `Ng_z`)
    /// * the hit's `u` and `v` coordinates
    /// * the hit's `primID`, `geomID` and `instID`
    ///
    /// must all be updated.
    ///
    /// Setting `ray_hit.hit.geomID` to the supplied `geom_id` signals an intersection.
    fn intersect(
        &self,
        geom_id: u32,
        prim_id: u32,
        ctx: &embree4_sys::RTCRayQueryContext,
        ray_hit: &mut embree4_sys::RTCRayHit,
    );
}

pub struct UserGeometry<T: UserGeometryImpl> {
    handle: embree4_sys::RTCGeometry,
    data: PhantomData<T>,
}

#[allow(clippy::missing_safety_doc)]
impl<T: UserGeometryImpl> UserGeometry<T> {
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
    pub fn try_new(device: &Device, data: &T) -> Result<Self> {
        let handle = unsafe {
            embree4_sys::rtcNewGeometry(device.handle, embree4_sys::RTCGeometryType::USER)
        };
        device_error_or(device, (), "Could not create user geometry")?;

        unsafe {
            embree4_sys::rtcSetGeometryUserPrimitiveCount(handle, 1);
        }
        device_error_or(device, (), "Could not set user geometry primitive count")?;

        let data_ptr = data as *const _ as _;

        unsafe {
            embree4_sys::rtcSetGeometryUserData(handle, data_ptr);
        }
        device_error_or(device, (), "Could not set user geometry data")?;

        unsafe {
            embree4_sys::rtcSetGeometryBoundsFunction(
                handle,
                Some(internal_bounds_fn::<T>),
                data_ptr,
            );
        };
        device_error_or(device, (), "Could not set user geometry bounds function")?;

        unsafe {
            embree4_sys::rtcSetGeometryIntersectFunction(handle, Some(internal_intersect_fn::<T>));
        }
        device_error_or(device, (), "Could not set user geometry intersect function")?;

        // unsafe {
        //     embree4_sys::rtcSetGeometryOccludedFunction(handle, Some(occluded_fn));
        // }
        // device_error_or(device, (), "Could not set user geometry occluded function")?;

        // unsafe {
        //     embree4_sys::rtcSetGeometryPointQueryFunction(
        //         handle,
        //         Some(internal_point_query_fn::<T>),
        //     )
        // }

        unsafe {
            embree4_sys::rtcCommitGeometry(handle);
        }
        device_error_or(device, (), "Could not commit user geometry")?;

        Ok(Self {
            handle,
            data: PhantomData,
        })
    }
}

impl<T: UserGeometryImpl> Geometry for UserGeometry<T> {
    fn geometry(&self) -> embree4_sys::RTCGeometry {
        self.handle
    }
}

impl<T: UserGeometryImpl> Drop for UserGeometry<T> {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseGeometry(self.handle);
        }
    }
}

unsafe extern "C" fn internal_bounds_fn<T: UserGeometryImpl>(
    args: *const embree4_sys::RTCBoundsFunctionArguments,
) {
    let args = *args;
    let geom_ptr = args.geometryUserPtr as *const T;
    let geom = ptr::read(geom_ptr);

    *args.bounds_o = geom.bounds();
}

unsafe extern "C" fn internal_intersect_fn<T: UserGeometryImpl>(
    args: *const embree4_sys::RTCIntersectFunctionNArguments,
) {
    let args = &*args;
    let geom_ptr = args.geometryUserPtr as *const T;
    let geom = ptr::read(geom_ptr);

    let rayhit_n = args.rayhit as *mut f32;

    let ray_n = rayhit_n;
    let hit_n = rayhit_n.add(12 * args.N as usize);

    let valid_ptr = args.valid as *const u32;
    let valid = std::slice::from_raw_parts(valid_ptr, args.N as usize);

    let context = &*(args.context as *const embree4_sys::RTCRayQueryContext);

    let n = args.N as usize;
    for (i, valid) in valid.iter().enumerate() {
        if *valid == 0 {
            continue;
        }

        let org_x = ray_n.add(offset(0, n, i));
        let org_y = ray_n.add(offset(1, n, i));
        let org_z = ray_n.add(offset(2, n, i));
        let tnear = ray_n.add(offset(3, n, i));

        let dir_x = ray_n.add(offset(4, n, i));
        let dir_y = ray_n.add(offset(5, n, i));
        let dir_z = ray_n.add(offset(6, n, i));
        let time = ray_n.add(offset(7, n, i));

        let tfar = ray_n.add(offset(8, n, i));
        let mask = ray_n.add(offset(9, n, i)) as *mut u32;
        let id = ray_n.add(offset(10, n, i)) as *mut u32;
        let flags = ray_n.add(offset(11, n, i)) as *mut u32;

        let ng_x = hit_n.add(offset(0, n, i));
        let ng_y = hit_n.add(offset(1, n, i));
        let ng_z = hit_n.add(offset(2, n, i));

        let u = hit_n.add(offset(3, n, i));
        let v = hit_n.add(offset(4, n, i));

        let prim_id = hit_n.add(offset(5, n, i)) as *mut u32;
        let geom_id = hit_n.add(offset(6, n, i)) as *mut u32;
        let inst_id = hit_n.add(offset(7, n, i)) as *mut u32;

        let mut ray_hit = RTCRayHit {
            ray: embree4_sys::RTCRay {
                org_x: *org_x,
                org_y: *org_y,
                org_z: *org_z,
                tnear: *tnear,
                dir_x: *dir_x,
                dir_y: *dir_y,
                dir_z: *dir_z,
                time: *time,
                tfar: *tfar,
                mask: *mask,
                id: *id,
                flags: *flags,
            },
            hit: embree4_sys::RTCHit {
                Ng_x: *ng_x,
                Ng_y: *ng_y,
                Ng_z: *ng_z,
                u: *u,
                v: *v,
                primID: *prim_id,
                geomID: *geom_id,
                instID: [*inst_id],
            },
        };

        geom.intersect(args.geomID, args.primID, context, &mut ray_hit);

        if ray_hit.hit.geomID != RTC_INVALID_GEOMETRY_ID {
            *tfar = ray_hit.ray.tfar;

            *ng_x = ray_hit.hit.Ng_x;
            *ng_y = ray_hit.hit.Ng_y;
            *ng_z = ray_hit.hit.Ng_z;

            *u = ray_hit.hit.u;
            *v = ray_hit.hit.v;

            *prim_id = ray_hit.hit.primID;
            *geom_id = ray_hit.hit.geomID;
            *inst_id = ray_hit.hit.instID[0];
        }
    }
}

#[inline(always)]
fn offset(offset: usize, n: usize, i: usize) -> usize {
    offset * n + i
}
