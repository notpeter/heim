use std::{
    ffi::CStr,
    io, mem,
    process::Command,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use heim_common::{
    prelude::*,
    units::{time, Time},
};

use crate::Arch;

#[derive(Debug)]
pub struct Platform {
    system: String,
    release: String,
    version: String,
    hostname: String,
    arch: Arch,
}

impl Platform {
    pub fn system(&self) -> &str {
        &self.system
    }

    pub fn release(&self) -> &str {
        &self.release
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn architecture(&self) -> Arch {
        self.arch
    }
}

pub async fn platform() -> Result<Platform> {
    unsafe {
        let mut uts = mem::MaybeUninit::<libc::utsname>::uninit();
        let result = libc::uname(uts.as_mut_ptr());

        if result != 0 {
            Err(Error::last_os_error().with_ffi("uname"))
        } else {
            let uts = uts.assume_init();
            let raw_arch = CStr::from_ptr(uts.machine.as_ptr()).to_string_lossy();
            let arch = Arch::from_str(&raw_arch).unwrap_or(Arch::Unknown);

            Ok(Platform {
                system: CStr::from_ptr(uts.sysname.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                release: CStr::from_ptr(uts.release.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                version: CStr::from_ptr(uts.version.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                hostname: CStr::from_ptr(uts.nodename.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                arch,
            })
        }
    }
}

fn kstat_value(name: &str) -> io::Result<f64> {
    let output = Command::new("kstat")
        .args(["-p", &format!("unix:0:system_misc:{name}")])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "kstat failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing kstat value"))?
        .parse::<f64>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub async fn boot_time() -> Result<Time> {
    kstat_value("boot_time")
        .map(Time::new::<time::second>)
        .map_err(Into::into)
}

pub async fn uptime() -> Result<Time> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?
        .as_secs_f64();
    let boot = kstat_value("boot_time")?;

    Ok(Time::new::<time::second>(now - boot))
}

#[derive(Debug)]
pub struct User {
    username: String,
}

impl User {
    pub fn username(&self) -> &str {
        self.username.as_str()
    }
}

pub async fn users() -> Result<impl Stream<Item = Result<User>>> {
    Ok(stream::empty())
}
