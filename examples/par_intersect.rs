use anyhow::Result;
use embree4_rs::{geometry::TriangleMeshGeometry, Device, Scene, SceneOptions};
use glam::Vec3;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

pub fn main() -> Result<()> {
    let device = Device::try_new(None)?;

    let mut rng = StdRng::seed_from_u64(0);

    let num_tris = 1_000_000;
    let mut vertices = Vec::with_capacity(3 * num_tris);
    let mut indices = Vec::with_capacity(num_tris);

    for i in 0..num_tris as u32 {
        let pos = 1_000.0 * (2.0 * rng.gen::<Vec3>() - 1.0);

        let p = pos + rng.gen::<Vec3>();
        let q = pos + rng.gen::<Vec3>();
        let r = pos + rng.gen::<Vec3>();

        vertices.push((p.x, p.y, p.z));
        vertices.push((q.x, q.y, q.z));
        vertices.push((r.x, r.y, r.z));

        indices.push((3 * i, 3 * i + 1, 3 * i + 2));
    }

    let mesh = TriangleMeshGeometry::try_new(&device, &vertices, &indices)?;
    let scene = Scene::try_new(
        &device,
        SceneOptions {
            build_quality: embree4_sys::RTCBuildQuality::HIGH,
            flags: embree4_sys::RTCSceneFlags::ROBUST,
        },
    )?;
    scene.attach_geometry(&mesh)?;
    let scene = scene.commit()?;

    let num_rays = 1_000_000;
    let rays: Vec<_> = (0..num_rays)
        .map(|_| {
            let origin = Vec3::ZERO;
            let direction = 2.0 * rng.gen::<Vec3>() - 1.0;

            embree4_sys::RTCRay {
                org_x: origin.x,
                org_y: origin.y,
                org_z: origin.z,
                dir_x: direction.x,
                dir_y: direction.y,
                dir_z: direction.z,
                ..Default::default()
            }
        })
        .collect();

    let t0 = std::time::Instant::now();
    let hits: usize = rays
        .into_par_iter()
        .map(|ray| match scene.intersect_1(ray).unwrap() {
            Some(_) => 1,
            None => 0,
        })
        .sum();
    let elapsed = t0.elapsed();
    let rays_per_sec = (num_rays as f32 / elapsed.as_secs_f32()) as usize;

    println!("Traced {} rays in {:?}", num_rays, elapsed);
    let frac_hits = hits as f32 / num_rays as f32;
    println!("  {} hits ({:.3}%)", hits, 100.0 * frac_hits);
    println!("  ({} rays/s)", rays_per_sec);

    Ok(())
}
