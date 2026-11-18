#![cfg(target_os = "illumos")]

use heim_common::prelude::*;
use heim_common::units::information;
use heim_net::os::illumos::NicExt;
use kstat::kstat_named::KstatNamedData;
use kstat::{KstatData, KstatReader};

fn value(stat: &KstatData, key: &str) -> u64 {
    match stat.data.get(key).unwrap() {
        KstatNamedData::DataUInt64(value) => *value,
        KstatNamedData::DataUInt32(value) => u64::from(*value),
        other => panic!("unexpected counter type: {:?}", other),
    }
}

#[heim_derive::test]
async fn counters_match_native_snapshots() -> Result<()> {
    let reader = KstatReader::new(Some("link"), None, None, Some("net"))?;
    let before = reader.read()?;
    let counters: Vec<_> = heim_net::io_counters().await?.try_collect().await?;
    let after = reader.read()?;
    assert_eq!(counters.len(), before.len());
    for counter in counters {
        let first = before
            .iter()
            .find(|s| s.name == counter.interface())
            .unwrap();
        let last = after
            .iter()
            .find(|s| s.name == counter.interface())
            .unwrap();
        for (key, observed) in &[
            ("rbytes64", counter.bytes_recv().get::<information::byte>()),
            ("obytes64", counter.bytes_sent().get::<information::byte>()),
            ("ipackets64", counter.packets_recv()),
            ("opackets64", counter.packets_sent()),
            ("ierrors", counter.errors_recv()),
            ("oerrors", counter.errors_sent()),
            ("norcvbuf", counter.drop_recv()),
        ] {
            assert!(
                (value(first, key)..=value(last, key)).contains(observed),
                "{} {}: {} not between native snapshots",
                counter.interface(),
                key,
                observed
            );
        }
        println!("{:?}", counter);
    }
    Ok(())
}

#[heim_derive::test]
async fn nic_includes_loopback() -> Result<()> {
    let interfaces: Vec<_> = heim_net::nic().await?.try_collect().await?;
    assert!(interfaces.iter().any(|nic| nic.is_loopback()));
    for nic in interfaces {
        assert!(!nic.name().is_empty());
        assert!(nic.index().is_some());
        let _ = (nic.broadcast(), nic.is_broadcast(), nic.is_point_to_point());
    }
    Ok(())
}
