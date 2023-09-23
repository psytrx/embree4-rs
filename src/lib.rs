mod device;
mod scene;

use anyhow::{bail, Result};

pub use device::*;
use embree4_sys::RTCDevice;
pub use scene::*;

fn device_error_raw(device: RTCDevice) -> Option<embree4_sys::RTCError> {
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
