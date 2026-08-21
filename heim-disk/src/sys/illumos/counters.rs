use std::ffi::OsStr;

use heim_common::prelude::*;
use heim_common::units::{information, Information};

#[derive(Debug)]
pub struct IoCounters;

impl IoCounters {
    pub fn device_name(&self) -> &OsStr {
        OsStr::new("")
    }

    pub fn read_count(&self) -> u64 {
        0
    }

    pub fn write_count(&self) -> u64 {
        0
    }

    pub fn read_bytes(&self) -> Information {
        Information::new::<information::byte>(0)
    }

    pub fn write_bytes(&self) -> Information {
        Information::new::<information::byte>(0)
    }
}

pub async fn io_counters() -> Result<impl Stream<Item = Result<IoCounters>>> {
    Ok(stream::empty())
}

pub async fn io_counters_physical() -> Result<impl Stream<Item = Result<IoCounters>>> {
    io_counters().await
}
