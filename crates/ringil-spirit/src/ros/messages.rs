//! Interface declarations.

use rosidl_runtime_rs::{Sequence, String as RosString};

/// Represents an Image frame.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub encoding: RosString,
    pub step: u32,
    pub data: Sequence<u8>,
}
