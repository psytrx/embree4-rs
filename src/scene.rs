use anyhow::{bail, Result};

use crate::Device;

pub struct Scene<'a> {
    device: &'a Device,
    handle: embree4_sys::RTCScene,
}

impl<'a> Scene<'a> {
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

    pub fn set_build_quality(&self, quality: SceneBuildQuality) -> Result<()> {
        let quality = quality.to_sys_enum();
        unsafe {
            embree4_sys::rtcSetSceneBuildQuality(self.handle, quality);
        }
        let error = unsafe { embree4_sys::rtcGetDeviceError(self.device.handle) };
        if error != embree4_sys::RTCError::RTC_ERROR_NONE {
            bail!("Failed to set scene build quality: {:?}", error);
        }
        Ok(())
    }

    pub fn set_flags(&self, flags: SceneFlags) -> Result<()> {
        let flags = flags.to_sys_flags();
        unsafe {
            embree4_sys::rtcSetSceneFlags(self.handle, flags);
        }
        let error = unsafe { embree4_sys::rtcGetDeviceError(self.device.handle) };
        if error != embree4_sys::RTCError::RTC_ERROR_NONE {
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
    pub fn to_sys_enum(&self) -> embree4_sys::RTCBuildQuality {
        match self {
            SceneBuildQuality::Low => embree4_sys::RTCBuildQuality::RTC_BUILD_QUALITY_LOW,
            SceneBuildQuality::Medium => embree4_sys::RTCBuildQuality::RTC_BUILD_QUALITY_MEDIUM,
            SceneBuildQuality::High => embree4_sys::RTCBuildQuality::RTC_BUILD_QUALITY_HIGH,
        }
    }
}

#[repr(u32)]
#[derive(PartialEq, Default)]
pub enum SceneFlags {
    #[default]
    None = 0,
    Dynamic = (1 << 0),
    Compact = (1 << 1),
    Robust = (1 << 2),
    FilterFunction = (1 << 3),
}

impl SceneFlags {
    pub fn to_sys_flags(&self) -> embree4_sys::RTCSceneFlags {
        let flags = self as *const _ as u32;
        embree4_sys::RTCSceneFlags(flags)
    }
}
