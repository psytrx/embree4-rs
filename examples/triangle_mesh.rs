use embree4_rs::{geometry::TriangleMeshGeometry, Device, Scene, SceneOptions};
use embree4_sys::{RTCBuildQuality, RTCSceneFlags};

use anyhow::Result;

fn main() -> Result<()> {
    let config = Some("verbose=1");
    let device = Device::try_new(config)?;

    let scene_options = SceneOptions {
        build_quality: RTCBuildQuality::HIGH,
        flags: RTCSceneFlags::ROBUST | RTCSceneFlags::COMPACT,
    };
    let scene = Scene::try_new(&device, scene_options)?;

    let vertices = [
        (-1.0, -1.0, 0.0),
        (1.0, -1.0, 0.0),
        (1.0, 1.0, 0.0),
        (-1.0, 1.0, 0.0),
    ];
    let indices = [(0, 1, 2), (2, 3, 0)];

    let tri_mesh = TriangleMeshGeometry::try_new(&device, &vertices, &indices)?;
    scene.attach_geometry(&tri_mesh)?;

    Ok(())
}
