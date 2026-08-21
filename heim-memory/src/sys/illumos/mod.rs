use heim_common::prelude::*;
use heim_common::units::{information, Information};

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct Memory {
    total: Information,
    free: Information,
}

impl Memory {
    pub fn total(&self) -> Information {
        self.total
    }

    pub fn available(&self) -> Information {
        self.free
    }

    pub fn free(&self) -> Information {
        self.free
    }
}

#[derive(Debug, Default, Clone)]
pub struct Swap;

impl Swap {
    pub fn total(&self) -> Information {
        Information::new::<information::byte>(0)
    }

    pub fn used(&self) -> Information {
        Information::new::<information::byte>(0)
    }

    pub fn free(&self) -> Information {
        Information::new::<information::byte>(0)
    }

    pub fn sin(&self) -> Option<Information> {
        None
    }

    pub fn sout(&self) -> Option<Information> {
        None
    }
}

pub async fn memory() -> Result<Memory> {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let pages = unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) };
    let free_pages = unsafe { libc::sysconf(libc::_SC_AVPHYS_PAGES) };

    if page_size < 0 {
        return Err(Error::last_os_error().with_sysconf(libc::_SC_PAGESIZE));
    }
    if pages < 0 {
        return Err(Error::last_os_error().with_sysconf(libc::_SC_PHYS_PAGES));
    }
    if free_pages < 0 {
        return Err(Error::last_os_error().with_sysconf(libc::_SC_AVPHYS_PAGES));
    }

    let page_size = page_size as u64;
    Ok(Memory {
        total: Information::new::<information::byte>(pages as u64 * page_size),
        free: Information::new::<information::byte>(free_pages as u64 * page_size),
    })
}

pub async fn swap() -> Result<Swap> {
    Ok(Swap)
}
