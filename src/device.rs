use std::ptr::null_mut;

use anyhow::{bail, Result};

use crate::device_error_raw;

pub struct Device {
    pub(crate) handle: embree4_sys::RTCDevice,
}

impl Device {
    /// Constructs a new `Device` using the provided configuration string.
    ///
    /// # Arguments
    /// * `config` - A string representing the configuration for the device. Can be an empty string.
    ///              See [rtcNewDevice](https://github.com/embree/embree/blob/master/doc/src/api/rtcNewDevice.md) for valid configuration values.
    ///
    /// # Returns
    /// A `Result` containing the created `Device` if successful, or an error if the device creation fails.
    ///
    /// # Examples
    /// ```
    /// use embree4_rs::Device;
    ///
    /// match Device::try_new(Some("verbose=3,start_threads=1")) {
    ///     Ok(device) => {
    ///         println!("Device created successfully!");
    ///         // Use the device...
    ///     },
    ///     Err(error) => println!("Could not create device: {}", error),
    /// }
    /// ```
    pub fn try_new(config: Option<&str>) -> Result<Self> {
        let handle = match config {
            None => unsafe { embree4_sys::rtcNewDevice(null_mut()) },
            Some(config) => unsafe {
                embree4_sys::rtcNewDevice(config.as_bytes() as *const _ as _)
            },
        };

        if handle.is_null() {
            let error = device_error_raw(null_mut());
            bail!("Failed to create device: {:?}", error);
        }

        Ok(Device { handle })
    }

    /// Returns the error code associated with the device, if any.
    ///
    /// # Returns
    /// `Some(error_code)` if there is an error associated with the device, otherwise `None`.
    pub fn error(&self) -> Option<embree4_sys::RTCError> {
        device_error_raw(self.handle)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseDevice(self.handle);
        }
    }
}

#[test]
fn try_new_valid_config() {
    let ok_device = Device::try_new(Some("verbose=0"));
    assert!(ok_device.is_ok());
}

#[test]
fn try_new_invalid_config() {
    let err_device = Device::try_new(Some("verbose=bruh"));
    assert!(err_device.is_err());
}

#[test]
fn try_new_no_config() {
    let ok_device = Device::try_new(None);
    assert!(ok_device.is_ok());
}
