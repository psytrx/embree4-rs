mod device;
pub mod geometry;
mod scene;

use anyhow::{bail, Result};

pub use device::*;
pub use scene::*;

fn device_error_raw(device: embree4_sys::RTCDevice) -> Option<embree4_sys::RTCError> {
    let err = unsafe { embree4_sys::rtcGetDeviceError(device) };
    if err != embree4_sys::RTCError::NONE {
        Some(err)
    } else {
        None
    }
}

fn device_error_or<T>(device: &Device, ok_value: T, message: &str) -> Result<T> {
    device_error_raw(device.handle)
        .map(|error| bail!("{}: {:?}", message, error))
        .unwrap_or(Ok(ok_value))
}
