use std::ffi::OsStr;
use std::io;

use heim_common::prelude::*;
use heim_common::units::{information, Information};
use kstat_rs::{Ctl, Data, Type};

#[derive(Debug)]
pub struct IoCounters {
    device: String,
    data: kstat_rs::Io,
}

impl IoCounters {
    pub fn device_name(&self) -> &OsStr {
        OsStr::new(&self.device)
    }
    pub fn read_count(&self) -> u64 {
        u64::from(self.data.reads)
    }
    pub fn write_count(&self) -> u64 {
        u64::from(self.data.writes)
    }
    pub fn read_bytes(&self) -> Information {
        Information::new::<information::byte>(self.data.nread)
    }
    pub fn write_bytes(&self) -> Information {
        Information::new::<information::byte>(self.data.nwritten)
    }
}

fn error(error: kstat_rs::Error) -> Error {
    match error {
        kstat_rs::Error::Io(error) => error.into(),
        other => io::Error::new(io::ErrorKind::InvalidData, other).into(),
    }
}

async fn counters(physical: bool) -> Result<impl Stream<Item = Result<IoCounters>>> {
    let entries = heim_runtime::spawn_blocking(move || -> Result<Vec<Result<IoCounters>>> {
        let ctl = Ctl::new().map_err(error)?;
        let mut entries = Vec::new();
        for mut stat in ctl.iter() {
            if stat.ks_type != Type::Io || !matches!(stat.ks_class, "disk" | "partition") {
                continue;
            }
            // Pool and file-backed device statistics are not physical disk counters.
            if physical
                && (stat.ks_class != "disk" || matches!(stat.ks_module, "zfs" | "zvol" | "lofi"))
            {
                continue;
            }
            let device = stat.ks_name.to_owned();
            match ctl.read(&mut stat) {
                Ok(Data::Io(data)) => entries.push(Ok(IoCounters { device, data })),
                // The device may have disappeared since kstat_open took its snapshot.
                Err(kstat_rs::Error::Io(ref e)) if e.raw_os_error() == Some(libc::ENXIO) => {
                    continue
                }
                Err(e) => entries.push(Err(error(e))),
                Ok(_) => entries.push(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Expected disk I/O kstat",
                )
                .into())),
            }
        }
        Ok(entries)
    })
    .await?;
    Ok(stream::iter(entries))
}

pub async fn io_counters() -> Result<impl Stream<Item = Result<IoCounters>>> {
    counters(false).await
}
pub async fn io_counters_physical() -> Result<impl Stream<Item = Result<IoCounters>>> {
    counters(true).await
}
