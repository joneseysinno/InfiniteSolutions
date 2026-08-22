//! The operating-system boundary (D18, bootstrap plan §3).
//!
//! Window, GPU device, OS input, the tick loop. Names graphics and windowing
//! crates; names **no** layer crate (D32). Input arrives as pending amends, never
//! as writes (D24).
//!
//! Module file: docs, `mod` declarations, and re-exports only (F-8).

mod device;
mod drive;
mod input;
mod window;

pub use device::Device;
pub use drive::drive;
pub use input::Input;
pub use window::Window;
