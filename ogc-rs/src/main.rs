//! # ogc-rs
//!
//! ``ogc-rs`` is a safe, idiomatic wrapper around ``ogc-sys``.
//!
//! ``ogc-rs`` provides many features from libogc such as:
//!
//! * ``network``: Provides TCP networking for the Wii.
//! * ``audio``: Provides functions for audio on the Wii.
//! * ``fs``: Provides functions for manipulating the filesystem on the Wii.
//! * ``console``: Provides console functions for the Wii.
//! * ``input``: Provides an interface for reading input from devices on the Wii.
//! * ``video``: Provides functions for video output on the Wii.
//! * ``gx``: Provides an opengl-like interface for rendering on the Wii.
//!
//! Features such as ``network``, ``fs`` and ``console`` have been merged into ``std`` to extend functionality.
//! However it is still possible to use these features if you require certain functionality missing from ``std``.

#![crate_type = "rlib"]
#![crate_name = "ogc"]

/// Custom Error Type for functions in this library.
pub mod error;
