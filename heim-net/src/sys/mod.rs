#[cfg(all(unix, not(target_os = "illumos")))]
mod unix;

#[cfg(all(unix, not(target_os = "illumos")))]
pub use self::unix::*;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;

        pub use self::linux::*;
    } else if #[cfg(target_os = "macos")] {
        mod macos;

        pub use self::macos::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;

        pub use self::windows::*;
    } else if #[cfg(target_os = "illumos")] {
        mod illumos;

        pub use self::illumos::*;
    }
}
