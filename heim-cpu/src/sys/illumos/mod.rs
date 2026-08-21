use std::io;

use heim_common::prelude::*;
use heim_common::units::{time, Time};

#[derive(Debug, Default, Clone)]
pub struct CpuTime {
    user: Time,
    system: Time,
    idle: Time,
}

impl CpuTime {
    pub fn user(&self) -> Time {
        self.user
    }

    pub fn system(&self) -> Time {
        self.system
    }

    pub fn idle(&self) -> Time {
        self.idle
    }
}

pub async fn logical_count() -> Result<u64> {
    let result = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };

    if result < 0 {
        Err(Error::last_os_error().with_sysconf(libc::_SC_NPROCESSORS_ONLN))
    } else {
        Ok(result as u64)
    }
}

pub async fn physical_count() -> Result<Option<u64>> {
    logical_count().await.map(Some)
}

pub async fn frequency() -> Result<CpuFrequency> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "CPU frequency is unsupported on illumos",
    )
    .into())
}

#[derive(Debug, Default)]
pub struct CpuFrequency;

impl CpuFrequency {
    pub fn current(&self) -> heim_common::units::Frequency {
        heim_common::units::Frequency::new::<heim_common::units::frequency::megahertz>(0)
    }

    pub fn min(&self) -> Option<heim_common::units::Frequency> {
        None
    }

    pub fn max(&self) -> Option<heim_common::units::Frequency> {
        None
    }
}

pub async fn stats() -> Result<CpuStats> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "CPU stats are unsupported on illumos",
    )
    .into())
}

#[derive(Debug, Default)]
pub struct CpuStats;

impl CpuStats {
    pub fn ctx_switches(&self) -> u64 {
        0
    }

    pub fn interrupts(&self) -> u64 {
        0
    }

    pub fn soft_interrupts(&self) -> u64 {
        0
    }

    pub fn syscalls(&self) -> u64 {
        0
    }
}

pub async fn time() -> Result<CpuTime> {
    Ok(CpuTime {
        user: Time::new::<time::second>(0.0),
        system: Time::new::<time::second>(0.0),
        idle: Time::new::<time::second>(0.0),
    })
}

pub async fn times() -> Result<impl Stream<Item = Result<CpuTime>>> {
    Ok(stream::empty())
}
