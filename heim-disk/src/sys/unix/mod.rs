#[cfg(not(any(target_os = "linux", target_os = "illumos")))] // Linux have it's own way
mod partitions;
mod usage;

#[cfg(not(any(target_os = "linux", target_os = "illumos")))]
mod bindings;

#[cfg(not(any(target_os = "linux", target_os = "illumos")))]
pub use self::partitions::*;
pub use self::usage::*;
