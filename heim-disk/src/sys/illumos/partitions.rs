use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
};

use heim_common::prelude::*;
use heim_runtime as rt;

use crate::FileSystem;

#[derive(Debug)]
pub struct Partition {
    device: String,
    mount_point: PathBuf,
    fs_type: FileSystem,
    options: String,
}

impl Partition {
    pub fn device(&self) -> Option<&OsStr> {
        Some(OsStr::new(self.device.as_str()))
    }

    pub fn mount_point(&self) -> &Path {
        self.mount_point.as_path()
    }

    pub fn file_system(&self) -> &FileSystem {
        &self.fs_type
    }

    pub fn options(&self) -> &str {
        self.options.as_str()
    }
}

impl FromStr for Partition {
    type Err = Error;

    fn from_str(line: &str) -> Result<Partition> {
        let mut parts = line.split_whitespace();
        let device = match parts.next() {
            Some(device) => device.to_string(),
            None => return Err(Error::missing_key("device", "/etc/mnttab")),
        };
        let mount_point = match parts.next() {
            Some(point) => PathBuf::from(point),
            None => return Err(Error::missing_key("mount point", "/etc/mnttab")),
        };
        let fs_type = match parts.next() {
            Some(fs) => FileSystem::from_str(fs)?,
            None => return Err(Error::missing_key("file-system type", "/etc/mnttab")),
        };
        let options = match parts.next() {
            Some(options) => options.to_string(),
            None => return Err(Error::missing_key("options", "/etc/mnttab")),
        };

        Ok(Partition {
            device,
            mount_point,
            fs_type,
            options,
        })
    }
}

pub async fn partitions() -> Result<impl Stream<Item = Result<Partition>>> {
    let lines = rt::fs::read_lines("/etc/mnttab").await?;
    let stream = lines
        .map_err(Error::from)
        .try_filter_map(|line| async move { Ok(Partition::from_str(&line).ok()) });

    Ok(stream)
}

pub async fn partitions_physical() -> Result<impl Stream<Item = Result<Partition>>> {
    let stream = partitions().await?;
    let stream = stream.try_filter_map(|partition| {
        if partition.file_system().is_physical() {
            future::ok(Some(partition))
        } else {
            future::ok(None)
        }
    });

    Ok(stream)
}
