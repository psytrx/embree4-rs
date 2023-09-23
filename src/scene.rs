use anyhow::{bail, Result};
use bitflags::bitflags;

use crate::Device;

pub struct Scene<'a> {
    device: &'a Device,
    handle: embree4_sys::RTCScene,
}

impl<'a> Scene<'a> {
    /// Constructs a new `Scene` instance from the given options.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the `Device` instance.
    /// * `options` - The options for creating the scene.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Scene` instance if successful, or an error if an error occurred.
    ///
    /// # Example
    ///
    /// ```
    /// use embree4_rs::*;
    ///
    /// let device = Device::try_new("").unwrap();
    /// let options = SceneOptions {
    ///     build_quality: SceneBuildQuality::High,
    ///     flags: SceneFlags::Compact | SceneFlags::Robust,
    /// };
    /// let scene = Scene::try_new(&device, options).unwrap();
    /// ```
    pub fn try_new(device: &'a Device, options: SceneOptions) -> Result<Self> {
        let handle = unsafe { embree4_sys::rtcNewScene(device.handle) };
        if handle.is_null() {
            let error = unsafe { embree4_sys::rtcGetDeviceError(device.handle) };
            bail!("Failed to create scene: {:?}", error);
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
    ///
    /// * `quality` - The build quality to set.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn set_build_quality(&self, quality: SceneBuildQuality) -> Result<()> {
        let quality = quality.to_sys_enum();
        unsafe {
            embree4_sys::rtcSetSceneBuildQuality(self.handle, quality);
        }
        let error = unsafe { embree4_sys::rtcGetDeviceError(self.device.handle) };
        if error != embree4_sys::RTCError::NONE {
            bail!("Failed to set scene build quality: {:?}", error);
        }
        Ok(())
    }

    /// Sets the flags of the scene.
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags to set.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn set_flags(&self, flags: SceneFlags) -> Result<()> {
        let flags = flags.to_sys_flags();
        unsafe {
            embree4_sys::rtcSetSceneFlags(self.handle, flags);
        }
        let error = unsafe { embree4_sys::rtcGetDeviceError(self.device.handle) };
        if error != embree4_sys::RTCError::NONE {
            panic!("Failed to set scene flags: {:?}", error);
        }
        Ok(())
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
    pub build_quality: SceneBuildQuality,
    pub flags: SceneFlags,
}

#[derive(PartialEq, Default)]
pub enum SceneBuildQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl SceneBuildQuality {
    /// Converts the `SceneBuildQuality` enum to the corresponding embree4_sys::RTCBuildQuality enum.
    pub fn to_sys_enum(&self) -> embree4_sys::RTCBuildQuality {
        match self {
            SceneBuildQuality::Low => embree4_sys::RTCBuildQuality::LOW,
            SceneBuildQuality::Medium => embree4_sys::RTCBuildQuality::MEDIUM,
            SceneBuildQuality::High => embree4_sys::RTCBuildQuality::HIGH,
        }
    }
}

bitflags! {
    #[derive(PartialEq, Default)]
    pub struct SceneFlags: u32 {
        const None = 0;
        const Dynamic = (1 << 0);
        const Compact = (1 << 1);
        const Robust = (1 << 2);
        const FilterFunction = (1 << 3);
    }
}

impl SceneFlags {
    /// Converts the `SceneFlags` enum to the corresponding embree4_sys::RTCSceneFlags enum.
    pub fn to_sys_flags(&self) -> embree4_sys::RTCSceneFlags {
        let flags = self as *const _ as u32;
        embree4_sys::RTCSceneFlags(flags)
    }
}
