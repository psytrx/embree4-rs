//! [![Crates.io](https://img.shields.io/crates/v/embree4-rs.svg)](https://crates.io/crates/embree4-rs)
//!
//! High-level wrapper for [Intel's Embree](https://www.embree.org/) 4 high-performance ray tracing
//! library.
//!
//! FFI Bindings from [embree4-sys](https://crates.io/crates/embree4-sys).
//!
//! A valid Embree installation is required. See
//! [Installation of Embree](https://github.com/embree/embree#installation-of-embree)
//! from the Embree docs.
//!
//! # Documentation
//!
//! Docs at [docs.rs](https://docs.rs/embree4-rs).
//!
//! See the [examples/](https://github.com/psytrx/embree4-rs/tree/main/examples) for a quick start
//! on how to use this crate.

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
