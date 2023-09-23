#![feature(pointer_byte_offsets)]

use std::{panic, ptr, slice};

use embree4_rs::{geometry::UserGeometry, Device, Scene};

use anyhow::Result;

fn main() -> Result<()> {
    let config = Some("verbose=1");
    let device = Device::try_new(config)?;
    let scene = Scene::try_new(&device, Default::default())?;

    // For user geometry, the data must outlive the scene.
    let sphere = Sphere {
        center: (0.0, 0.0, 4.0),
        radius: 1.0,
    };
    let geom = UserGeometry::try_new(&device, &sphere, bounds_fn, intersect_fn, occluded_fn)?;

    scene.attach_geometry(&geom)?;

    let _scene = scene.commit()?;

    Ok(())
}

struct Sphere {
    center: (f32, f32, f32),
    radius: f32,
}

impl Sphere {
    pub fn oops(&self) {
        panic!("oops");
    }
}

unsafe extern "C" fn bounds_fn(args: *const embree4_sys::RTCBoundsFunctionArguments) {
    let args = *args;
    let sphere = ptr::read(args.geometryUserPtr as *const Sphere);

    sphere.oops();

    let min = (
        sphere.center.0 - sphere.radius,
        sphere.center.1 - sphere.radius,
        sphere.center.2 - sphere.radius,
    );
    let max = (
        sphere.center.0 + sphere.radius,
        sphere.center.1 + sphere.radius,
        sphere.center.2 + sphere.radius,
    );

    let mut bounds = *args.bounds_o;
    bounds.lower_x = min.0;
    bounds.lower_y = min.1;
    bounds.lower_z = min.2;
    bounds.upper_x = max.0;
    bounds.upper_y = max.1;
    bounds.upper_z = max.2;
}

unsafe extern "C" fn intersect_fn(args: *const embree4_sys::RTCIntersectFunctionNArguments) {
    if true {
        panic!("oops");
    }

    let args = *args;
    let sphere = &*(args.geometryUserPtr as *const Sphere);

    let rayhit_n = args.rayhit;
    #[allow(clippy::erasing_op)]
    let ray_n = &(rayhit_n.byte_add(0 * args.N as usize) as *const _ as *mut embree4_sys::RTCRayN);
    let hit_n =
        &(rayhit_n.byte_add(4 * 12 * args.N as usize) as *const _ as *mut embree4_sys::RTCHitN);

    let valid_ptr = args.valid as *const u32;
    let valid = slice::from_raw_parts(valid_ptr, args.N as usize);

    let context = &*(args.context as *const embree4_sys::RTCRayQueryContext);

    for i in 0..args.N {
        if valid[i as usize] == 0 {
            continue;
        }

        #[allow(clippy::erasing_op)]
        let ox = ray_n.byte_add((4 * (0 * args.N + i)) as usize) as *const f32;
        let oy = ray_n.byte_add((4 * (args.N + i)) as usize) as *const f32;
        let oz = ray_n.byte_add((4 * (2 * args.N + i)) as usize) as *const f32;
        let _tnear = ray_n.byte_add((4 * (3 * args.N + i)) as usize) as *const f32;

        let dx = ray_n.byte_add((4 * (4 * args.N + i)) as usize) as *const f32;
        let dy = ray_n.byte_add((4 * (5 * args.N + i)) as usize) as *const f32;
        let dz = ray_n.byte_add((4 * (6 * args.N + i)) as usize) as *const f32;
        let _time = *(ray_n.add((4 * (7 * args.N + i)) as usize) as *const f32);

        let tfar = ray_n.byte_add((4 * (8 * args.N + i)) as usize) as *mut f32;
        let _mask = ray_n.byte_add((4 * (9 * args.N + i)) as usize) as *mut u32;
        let _id = ray_n.byte_add((4 * (10 * args.N + i)) as usize) as *mut u32;
        let _flags = ray_n.byte_add((4 * (11 * args.N + i)) as usize) as *mut u32;

        #[allow(clippy::erasing_op)]
        let _ng_x = hit_n.byte_add((4 * (0 * args.N + i)) as usize) as *mut f32;
        let _ng_y = hit_n.byte_add((4 * (args.N + i)) as usize) as *mut f32;
        let _ng_z = hit_n.byte_add((4 * (2 * args.N + i)) as usize) as *mut f32;

        let _u = hit_n.byte_add((4 * (3 * args.N + i)) as usize) as *mut f32;
        let _v = hit_n.byte_add((4 * (4 * args.N + i)) as usize) as *mut f32;

        let prim_id = hit_n.byte_add((4 * (5 * args.N + i)) as usize) as *mut u32;
        let geom_id = hit_n.byte_add((4 * (6 * args.N + i)) as usize) as *mut u32;
        let inst_id = hit_n.byte_add((4 * (7 * args.N + i)) as usize) as *mut u32;

        let origin = (*ox, *oy, *oz);
        let direction = (*dx, *dy, *dz);
        if let Some(t) = ray_sphere_intersect(sphere.center, sphere.radius, origin, direction) {
            *tfar = t;
            *inst_id = context.instID[0];
            *geom_id = args.geomID;
            *prim_id = args.primID;
        }
    }
}

unsafe extern "C" fn occluded_fn(_args: *const embree4_sys::RTCOccludedFunctionNArguments) {
    todo!("not implemented for brevity")
}

fn ray_sphere_intersect(
    center: (f32, f32, f32),
    r: f32,
    origin: (f32, f32, f32),
    direction: (f32, f32, f32),
) -> Option<f32> {
    let ox_cx = origin.0 - center.0;
    let oy_cy = origin.1 - center.1;
    let oz_cz = origin.2 - center.2;

    let a = direction.0 * direction.0 + direction.1 * direction.1 + direction.2 * direction.2;
    let b = 2.0 * (ox_cx * direction.0 + oy_cy * direction.1 + oz_cz * direction.2);
    let c = ox_cx * ox_cx + oy_cy * oy_cy + oz_cz * oz_cz - r * r;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        None
    } else {
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        if t1 >= 0.0 || t2 >= 0.0 {
            Some(t1.min(t2))
        } else {
            None // The sphere is behind the ray origin
        }
    }
}
