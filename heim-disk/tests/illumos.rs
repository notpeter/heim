#![cfg(target_os = "illumos")]

use std::collections::BTreeMap;
use std::ffi::CStr;
use std::mem::MaybeUninit;

use heim_common::prelude::*;
use heim_common::units::information;
use heim_disk::os::unix::UsageExt;
use kstat_rs::{Ctl, Data, Type};

fn disk_stats() -> BTreeMap<String, kstat_rs::Io> {
    let ctl = Ctl::new().unwrap();
    ctl.iter()
        .filter_map(|mut stat| {
            if stat.ks_type != Type::Io || !matches!(stat.ks_class, "disk" | "partition") {
                return None;
            }
            let name = stat.ks_name.to_owned();
            match ctl.read(&mut stat).unwrap() {
                Data::Io(data) => Some((name, data)),
                _ => panic!("expected I/O data"),
            }
        })
        .collect()
}

#[heim_derive::test]
async fn counters_match_native_snapshots() -> Result<()> {
    let before = disk_stats();
    let counters: Vec<_> = heim_disk::io_counters().await?.try_collect().await?;
    let after = disk_stats();
    assert_eq!(before.len(), counters.len());
    assert!(!counters.is_empty());
    for counter in counters {
        let name = counter.device_name().to_str().unwrap();
        let first = before.get(name).unwrap();
        let last = after.get(name).unwrap();
        for (low, observed, high) in &[
            (
                first.nread,
                counter.read_bytes().get::<information::byte>(),
                last.nread,
            ),
            (
                first.nwritten,
                counter.write_bytes().get::<information::byte>(),
                last.nwritten,
            ),
            (
                u64::from(first.reads),
                counter.read_count(),
                u64::from(last.reads),
            ),
            (
                u64::from(first.writes),
                counter.write_count(),
                u64::from(last.writes),
            ),
        ] {
            assert!(
                (low..=high).contains(&observed),
                "{} counter outside native snapshots",
                name
            );
        }
        println!("{:?}", counter);
    }
    let physical: Vec<_> = heim_disk::io_counters_physical()
        .await?
        .try_collect()
        .await?;
    let ctl = Ctl::new().unwrap();
    for counter in physical {
        assert!(ctl
            .iter()
            .any(|s| s.ks_name == counter.device_name().to_str().unwrap()
                && s.ks_class == "disk"
                && !matches!(s.ks_module, "zfs" | "zvol" | "lofi")));
    }
    Ok(())
}

fn root_stat() -> libc::statvfs {
    let mut raw = MaybeUninit::uninit();
    assert_eq!(
        unsafe { libc::statvfs(b"/\0".as_ptr().cast(), raw.as_mut_ptr()) },
        0
    );
    unsafe { raw.assume_init() }
}

#[heim_derive::test]
async fn root_mount_and_usage_match_native() -> Result<()> {
    let before = root_stat();
    let usage = heim_disk::usage("/").await?;
    let after = root_stat();
    for (a, actual, b) in &[
        (
            before.f_blocks * before.f_frsize,
            usage.total().get::<information::byte>(),
            after.f_blocks * after.f_frsize,
        ),
        (
            before.f_bavail * before.f_frsize,
            usage.free().get::<information::byte>(),
            after.f_bavail * after.f_frsize,
        ),
        (
            (before.f_blocks - before.f_bfree) * before.f_frsize,
            usage.used().get::<information::byte>(),
            (after.f_blocks - after.f_bfree) * after.f_frsize,
        ),
    ] {
        assert!((a.min(b)..=a.max(b)).contains(&actual));
    }
    assert_eq!(
        usage.flags().bits(),
        after.f_flag & (libc::ST_RDONLY | libc::ST_NOSUID)
    );
    let partitions: Vec<_> = heim_disk::partitions().await?.try_collect().await?;
    let root = partitions
        .iter()
        .find(|p| p.mount_point() == std::path::Path::new("/"))
        .unwrap();
    let fs = unsafe { CStr::from_ptr(after.f_basetype.as_ptr()) }
        .to_str()
        .unwrap();
    assert_eq!(root.file_system().as_str(), fs);
    assert!(root.device().is_some());
    println!("{:?} {:?}", root, usage);
    let physical: Vec<_> = heim_disk::partitions_physical()
        .await?
        .try_collect()
        .await?;
    assert!(physical.iter().all(|p| p.file_system().is_physical()));
    assert!(physical
        .iter()
        .any(|p| p.mount_point() == root.mount_point()));
    Ok(())
}

#[test]
fn native_filesystem_classification() {
    for name in &["zfs", "ufs", "pcfs", "hsfs"] {
        let fs: heim_disk::FileSystem = name.parse().unwrap();
        assert!(fs.is_physical());
        assert_eq!(fs.as_str(), *name);
    }
    for name in &["tmpfs", "proc", "devfs", "lofs", "nfs"] {
        assert!(!name.parse::<heim_disk::FileSystem>().unwrap().is_physical());
    }
}
