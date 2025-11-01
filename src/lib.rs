//! TS3 Security Level Calculator with CUDA acceleration
//!
//! This library provides CPU and GPU-accelerated SHA1 hashing
//! for calculating TeamSpeak 3 identity security levels.

#![deny(unsafe_code)]

pub mod hashers;
pub mod helpers;
pub mod identity;
pub mod level_improver;

// Re-export commonly used items
pub use hashers::{CpuHasher, CudaHasher};
pub use identity::Ts3Identity;
pub use level_improver::{LevelImprover, SecurityLevelHasher};
