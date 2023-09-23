use embree4_rs::{geometry::TriangleMeshGeometry, Device, Scene};

use anyhow::Result;

fn main() -> Result<()> {
    let config = Some("verbose=1");
    let device = Device::try_new(config)?;
    let scene = Scene::try_new(&device, Default::default())?;

    // Construct a quad from two triangles:
    // 3--2
    // | /|
    // |/ |
    // 0--1

    let vertices = [
        (-1.0, -1.0, 0.0),
        (1.0, -1.0, 0.0),
        (1.0, 1.0, 0.0),
        (-1.0, 1.0, 0.0),
    ];
    let indices = [(0, 1, 2), (2, 3, 0)];

    let tri_mesh = TriangleMeshGeometry::try_new(&device, &vertices, &indices)?;
    scene.attach_geometry(&tri_mesh)?;

    let _scene = scene.commit();

    Ok(())
}
