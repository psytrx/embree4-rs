#![feature(pointer_byte_offsets)]

use std::f32::{consts::PI, INFINITY};

use embree4_rs::{
    geometry::{UserGeometry, UserGeometryImpl},
    Device, Scene,
};

use anyhow::Result;
use embree4_sys::RTCRay;
use glam::{vec3, Vec3};

fn main() -> Result<()> {
    let config = Some("verbose=1");
    let device = Device::try_new(config)?;
    let scene = Scene::try_new(&device, Default::default())?;

    // For user geometry, the data must outlive the scene.
    let sphere = Sphere {
        center: vec3(0.0, 0.0, 5.0),
        radius: 1.0,
    };
    let geom = UserGeometry::try_new(&device, &sphere)?;

    scene.attach_geometry(&geom)?;

    let scene = scene.commit()?;

    let origin = Vec3::ZERO;
    let width = 32;
    let height = 18;

    let rays = width * height;
    let mut hits = 0;

    for x in 0..width {
        for y in 0..height {
            let u = (x as f32 + 0.5) / width as f32;
            let v = (y as f32 + 0.5) / height as f32;

            let target = vec3(u * 2.0 - 1.0, v * 2.0 - 1.0, 5.0);
            let direction = target - origin;

            let ray = RTCRay {
                org_x: origin.x,
                org_y: origin.y,
                org_z: origin.z,
                tnear: 0.0,
                dir_x: direction.x,
                dir_y: direction.y,
                dir_z: direction.z,
                time: 0.0,
                tfar: INFINITY,
                mask: (-1_i32) as u32,
                id: 0,
                flags: 0,
            };

            let hit = scene.intersect_1(ray)?;

            if hit.is_some() {
                hits += 1
            }
        }
    }

    let hit_fraction = hits as f32 / rays as f32;
    let approx_pi = hit_fraction * 4.0;
    println!("Hit fraction: {}", hit_fraction);
    println!("Approximated pi: {}", approx_pi);
    let err = (approx_pi - PI).abs();
    println!("Error: {}", err);

    Ok(())
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl UserGeometryImpl for Sphere {
    fn bounds(&self) -> embree4_sys::RTCBounds {
        embree4_sys::RTCBounds {
            lower_x: self.center.x - self.radius,
            lower_y: self.center.y - self.radius,
            lower_z: self.center.z - self.radius,
            align0: Default::default(),
            upper_x: self.center.x + self.radius,
            upper_y: self.center.y + self.radius,
            upper_z: self.center.z + self.radius,
            align1: Default::default(),
        }
    }

    fn intersect(
        &self,
        geom_id: u32,
        prim_id: u32,
        ctx: &embree4_sys::RTCRayQueryContext,
        ray_hit: &mut embree4_sys::RTCRayHit,
    ) {
        let o = vec3(ray_hit.ray.org_x, ray_hit.ray.org_y, ray_hit.ray.org_z);
        let d = vec3(ray_hit.ray.dir_x, ray_hit.ray.dir_y, ray_hit.ray.dir_z);
        let oc = o - self.center;

        let a = d.dot(d);
        let b = 2.0 * oc.dot(d);
        let c = oc.dot(oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return;
        }

        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        let t = t1.min(t2);
        ray_hit.ray.tfar = t;

        let n = (o + t * d - self.center).normalize();
        ray_hit.hit.Ng_x = n.x;
        ray_hit.hit.Ng_y = n.y;
        ray_hit.hit.Ng_z = n.z;

        // calculate uv coordinates
        let p = o + t * d;
        let phi = p.z.atan2(p.x);
        let theta = p.y.asin();

        let u = 1.0 - (phi + PI) / (2.0 * PI);
        ray_hit.hit.u = u;

        let v = (theta + PI / 2.0) / PI;
        ray_hit.hit.v = v;

        ray_hit.hit.instID = ctx.instID;
        ray_hit.hit.geomID = geom_id;
        ray_hit.hit.primID = prim_id;
    }
}
