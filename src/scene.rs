use anyhow::{bail, Result};

use crate::{device_error_or, device_error_raw, geometry::Geometry, Device};

pub struct Scene<'a> {
    device: &'a Device,
    handle: embree4_sys::RTCScene,
}

impl<'a> Scene<'a> {
    /// Constructs a new `Scene` instance from the given options.
    ///
    /// # Arguments
    /// * `device` - A reference to the `Device` instance.
    /// * `options` - The options for creating the scene.
    ///
    /// # Returns
    /// A `Result` containing the `Scene` instance if successful, or an error if an error occurred.
    ///
    /// # Example
    /// ```
    /// use embree4_rs::*;
    /// use embree4_sys::*;
    ///
    /// let device = Device::try_new(None).unwrap();
    /// let options = SceneOptions {
    ///     build_quality: RTCBuildQuality::HIGH,
    ///     flags: RTCSceneFlags::COMPACT | RTCSceneFlags::ROBUST,
    /// };
    /// let scene = Scene::try_new(&device, options).unwrap();
    /// ```
    pub fn try_new(device: &'a Device, options: SceneOptions) -> Result<Self> {
        let handle = unsafe { embree4_sys::rtcNewScene(device.handle) };

        if handle.is_null() {
            let error = device_error_raw(device.handle);
            bail!("Could not create scene: {:?}", error);
        }

        let scene = Scene { device, handle };

        if options.build_quality != Default::default() {
            scene.set_build_quality(options.build_quality)?;
        }

        if options.flags != Default::default() {
            scene.set_flags(options.flags)?;
        }

        Ok(scene)
    }

    /// Sets the build quality of the scene.
    ///
    /// # Arguments
    /// * `quality` - The build quality to set.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn set_build_quality(&self, quality: embree4_sys::RTCBuildQuality) -> Result<()> {
        unsafe {
            embree4_sys::rtcSetSceneBuildQuality(self.handle, quality);
        }
        device_error_or(self.device, (), "Could not set scene build quality")
    }

    /// Sets the flags of the scene.
    ///
    /// # Arguments
    /// * `flags` - The flags to set.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn set_flags(&self, flags: embree4_sys::RTCSceneFlags) -> Result<()> {
        unsafe {
            embree4_sys::rtcSetSceneFlags(self.handle, flags);
        }
        device_error_or(self.device, (), "Could not set scene flags")
    }

    /// Attaches the given geometry to the scene.
    ///
    /// # Arguments
    /// * `geometry` - A reference to the `Geometry` instance to attach.
    ///
    /// # Returns
    /// * A `Result` containing the geometry ID if successful, or an error if an error occurred.
    pub fn attach_geometry(&self, geometry: &impl Geometry) -> Result<u32> {
        let geom_id = unsafe { embree4_sys::rtcAttachGeometry(self.handle, geometry.geometry()) };
        device_error_or(self.device, geom_id, "Could not attach geometry")
    }

    /// Commits the scene.
    ///
    /// # Returns
    /// A `Result` containing the `CommittedScene` instance if successful, or an error if an error occurred.
    ///
    /// # Example
    /// ```
    /// use embree4_rs::*;
    /// use embree4_sys::*;
    ///
    /// let device = Device::try_new(None).unwrap();
    /// let options = Default::default();
    /// let scene = Scene::try_new(&device, options).unwrap();
    /// let scene = scene.commit().unwrap();
    /// ```
    pub fn commit(&self) -> Result<CommittedScene> {
        unsafe {
            embree4_sys::rtcCommitScene(self.handle);
        }
        device_error_or(
            self.device,
            CommittedScene { scene: self },
            "Could not commit scene",
        )
    }
}

impl<'a> Drop for Scene<'a> {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseScene(self.handle);
        }
    }
}

#[derive(Default)]
pub struct SceneOptions {
    pub build_quality: embree4_sys::RTCBuildQuality,
    pub flags: embree4_sys::RTCSceneFlags,
}

pub struct CommittedScene<'a> {
    scene: &'a Scene<'a>,
}

impl<'a> CommittedScene<'a> {
    pub fn intersect_1(&self, ray: embree4_sys::RTCRay) -> Result<Option<embree4_sys::RTCRayHit>> {
        let mut ray_hit = embree4_sys::RTCRayHit {
            ray,
            hit: embree4_sys::RTCHit {
                Ng_x: f32::default(),
                Ng_y: f32::default(),
                Ng_z: f32::default(),
                u: f32::default(),
                v: f32::default(),
                primID: u32::default(),
                geomID: embree4_sys::RTC_INVALID_GEOMETRY_ID,
                instID: [embree4_sys::RTC_INVALID_GEOMETRY_ID],
            },
        };

        unsafe {
            embree4_sys::rtcIntersect1(self.scene.handle, &mut ray_hit, std::ptr::null_mut());
        }
        device_error_or(self.scene.device, (), "Could not intersect ray")?;

        Ok(
            if ray_hit.hit.geomID != embree4_sys::RTC_INVALID_GEOMETRY_ID {
                Some(ray_hit)
            } else {
                None
            },
        )
    }
}
