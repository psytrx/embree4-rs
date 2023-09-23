use anyhow::{bail, Result};

pub struct Device {
    pub(crate) handle: embree4_sys::RTCDevice,
}

impl Device {
    /// Tries to create a new `Device` using the provided configuration string.
    ///
    /// # Arguments
    ///
    /// * `config` - A string representing the configuration for the device.
    ///              See [rtcNewDevice](https://github.com/embree/embree/blob/master/doc/src/api/rtcNewDevice.md) for valid configuration values.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the created `Device` if successful, or an error if the device creation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use embree4_rs::Device;
    ///
    /// match Device::try_new("verbose=3,start_threads=1") {
    ///     Ok(device) => {
    ///         println!("Device created successfully!");
    ///         // Use the device...
    ///     },
    ///     Err(error) => println!("Could not create device: {}", error),
    /// }
    /// ```
    pub fn try_new(config: &str) -> Result<Self> {
        let handle = unsafe { embree4_sys::rtcNewDevice(config.as_bytes() as *const _ as _) };

        if handle.is_null() {
            let error = unsafe { embree4_sys::rtcGetDeviceError(handle) };
            bail!("Failed to create device: {:?}", error);
        } else {
            Ok(Device { handle })
        }
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
fn try_new_result() {
    let ok_device = Device::try_new("verbose=0");
    assert!(ok_device.is_ok());

    let err_device = Device::try_new("verbose=bruh");
    assert!(err_device.is_err());
}
