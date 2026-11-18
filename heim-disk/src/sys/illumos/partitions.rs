use std::ffi::{CStr, OsStr, OsString};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;

use heim_common::prelude::*;

use crate::FileSystem;

// <sys/mnttab.h>; these definitions are not exposed by libc on illumos.
#[repr(C)]
struct MntTab {
    special: *mut libc::c_char,
    mount_point: *mut libc::c_char,
    fs_type: *mut libc::c_char,
    options: *mut libc::c_char,
    time: *mut libc::c_char,
}

extern "C" {
    fn getmntent(file: *mut libc::FILE, entry: *mut MntTab) -> libc::c_int;
}

struct MountTable(*mut libc::FILE);

impl Drop for MountTable {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::fclose(self.0);
        }
    }
}

#[derive(Debug)]
pub struct Partition {
    device: Option<OsString>,
    mount_point: PathBuf,
    fs: FileSystem,
}

impl Partition {
    pub fn device(&self) -> Option<&OsStr> {
        self.device.as_deref()
    }
    pub fn mount_point(&self) -> &Path {
        &self.mount_point
    }
    pub fn file_system(&self) -> &FileSystem {
        &self.fs
    }
}

fn read_partitions() -> Result<Vec<Partition>> {
    let file = unsafe { libc::fopen(b"/etc/mnttab\0".as_ptr().cast(), b"r\0".as_ptr().cast()) };
    if file.is_null() {
        return Err(Error::last_os_error().with_file("/etc/mnttab"));
    }
    let table = MountTable(file);
    let mut partitions = Vec::new();
    loop {
        let mut entry = MntTab {
            special: ptr::null_mut(),
            mount_point: ptr::null_mut(),
            fs_type: ptr::null_mut(),
            options: ptr::null_mut(),
            time: ptr::null_mut(),
        };
        match unsafe { getmntent(table.0, &mut entry) } {
            0 => {
                if entry.mount_point.is_null() || entry.fs_type.is_null() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Incomplete mount entry",
                    )
                    .into());
                }
                // Copy fields before the next getmntent call can reuse their storage.
                let device = if entry.special.is_null() {
                    None
                } else {
                    Some(
                        OsStr::from_bytes(unsafe { CStr::from_ptr(entry.special) }.to_bytes())
                            .to_owned(),
                    )
                };
                let mount_point = PathBuf::from(OsStr::from_bytes(
                    unsafe { CStr::from_ptr(entry.mount_point) }.to_bytes(),
                ));
                let fs = unsafe { CStr::from_ptr(entry.fs_type) }
                    .to_string_lossy()
                    .parse()?;
                partitions.push(Partition {
                    device,
                    mount_point,
                    fs,
                });
            }
            -1 => {
                if unsafe { libc::ferror(table.0) } != 0 {
                    return Err(Error::last_os_error().with_file("/etc/mnttab"));
                }
                break;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Malformed /etc/mnttab entry",
                )
                .into())
            }
        }
    }
    Ok(partitions)
}

pub async fn partitions() -> Result<impl Stream<Item = Result<Partition>>> {
    let entries = heim_runtime::spawn_blocking(read_partitions).await?;
    Ok(stream::iter(entries.into_iter().map(Ok)))
}

pub async fn partitions_physical() -> Result<impl Stream<Item = Result<Partition>>> {
    Ok(partitions()
        .await?
        .try_filter(|p| future::ready(p.file_system().is_physical())))
}
