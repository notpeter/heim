use std::io;

use heim_common::prelude::*;
use heim_common::units::{information, Information};
use kstat::kstat_named::KstatNamedData;
use kstat::{KstatData, KstatReader};

#[derive(Debug)]
pub struct IoCounters {
    interface: String,
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
    rx_errors: u64,
    tx_errors: u64,
    rx_drop: u64,
}

impl IoCounters {
    pub fn interface(&self) -> &str {
        &self.interface
    }
    pub fn bytes_sent(&self) -> Information {
        Information::new::<information::byte>(self.tx_bytes)
    }
    pub fn bytes_recv(&self) -> Information {
        Information::new::<information::byte>(self.rx_bytes)
    }
    pub fn packets_sent(&self) -> u64 {
        self.tx_packets
    }
    pub fn packets_recv(&self) -> u64 {
        self.rx_packets
    }
    pub fn errors_sent(&self) -> u64 {
        self.tx_errors
    }
    pub fn errors_recv(&self) -> u64 {
        self.rx_errors
    }
    pub fn drop_recv(&self) -> u64 {
        self.rx_drop
    }

    fn from_kstat(stat: KstatData) -> Result<Self> {
        Ok(Self {
            rx_bytes: counter(&stat, "rbytes64", Some("rbytes"))?,
            tx_bytes: counter(&stat, "obytes64", Some("obytes"))?,
            rx_packets: counter(&stat, "ipackets64", Some("ipackets"))?,
            tx_packets: counter(&stat, "opackets64", Some("opackets"))?,
            rx_errors: counter(&stat, "ierrors", None)?,
            tx_errors: counter(&stat, "oerrors", None)?,
            rx_drop: counter(&stat, "norcvbuf", None)?,
            interface: stat.name,
        })
    }
}

// Prefer the 64-bit counter; older providers may expose only its 32-bit alias.
// Missing or malformed counters are errors, not fabricated zero measurements.
fn counter(stat: &KstatData, name: &'static str, fallback: Option<&str>) -> Result<u64> {
    let value = stat
        .data
        .get(name)
        .or_else(|| fallback.and_then(|key| stat.data.get(key)));
    match value {
        Some(KstatNamedData::DataUInt64(value)) => Ok(*value),
        Some(KstatNamedData::DataUInt32(value)) => Ok(u64::from(*value)),
        None => Err(Error::missing_key(
            name,
            format!("kstat link:{}:{}", stat.instance, stat.name),
        )),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "kstat link:{}:{}: {} is not an unsigned counter",
                stat.instance, stat.name, name
            ),
        )
        .into()),
    }
}

pub async fn io_counters() -> Result<impl Stream<Item = Result<IoCounters>>> {
    // Keep the libkstat handle and all blocking reads on the worker thread.
    let counters = heim_runtime::spawn_blocking(|| -> Result<Vec<Result<IoCounters>>> {
        let reader = KstatReader::new(Some("link"), None, None, Some("net"))?;
        Ok(reader
            .read()?
            .into_iter()
            .map(IoCounters::from_kstat)
            .collect())
    })
    .await?;
    Ok(stream::iter(counters))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat() -> KstatData {
        let mut data = std::collections::HashMap::new();
        for name in &[
            "rbytes", "obytes", "ipackets", "opackets", "ierrors", "oerrors", "norcvbuf",
        ] {
            let _ = data.insert((*name).to_owned(), KstatNamedData::DataUInt32(7));
        }
        KstatData {
            class: "net".into(),
            module: "link".into(),
            instance: 0,
            name: "test0".into(),
            snaptime: 0,
            crtime: 0,
            data,
        }
    }

    #[test]
    fn prefers_wide_counters_and_preserves_values() {
        let mut stat = stat();
        let _ = stat
            .data
            .insert("rbytes64".into(), KstatNamedData::DataUInt64(1 << 40));
        let counters = IoCounters::from_kstat(stat).unwrap();
        assert_eq!(counters.bytes_recv().get::<information::byte>(), 1 << 40);
        assert_eq!(counters.bytes_sent().get::<information::byte>(), 7);
        assert_eq!(counters.packets_recv(), 7);
        assert_eq!(counters.drop_recv(), 7);
        assert_eq!(counters.interface(), "test0");
    }

    #[test]
    fn missing_and_invalid_counters_are_errors() {
        let mut stat = stat();
        let _ = stat.data.remove("norcvbuf");
        assert!(IoCounters::from_kstat(stat).is_err());
        let mut stat = self::stat();
        let _ = stat
            .data
            .insert("rbytes64".into(), KstatNamedData::DataInt64(-1));
        assert!(IoCounters::from_kstat(stat).is_err());
    }
}
