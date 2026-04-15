pub(crate) mod mimalloc;
pub(crate) mod perf;
pub mod resource;

pub(crate) use mimalloc::Mimalloc;
pub(crate) use perf::Perf;
pub use resource::Resource;

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct Global {
    pub date: u128,
    pub config: crate::Config,
    pub output: Process,
}

#[derive(Deserialize, Serialize)]
pub struct Process {
    pub index: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perf: Option<perf::Report>,
    pub mimalloc: Mimalloc,
    pub memory_key_value: u64,
    pub thread: Vec<Thread>,
}

#[derive(Deserialize, Serialize)]
pub struct Thread {
    pub id: usize,
    pub core: usize,
    pub time: u128,
    pub operation_count: u64,
    pub index: serde_json::Value,

    pub latency_get: Histogram,
    pub latency_update: Histogram,
    pub latency_insert: Histogram,
}

#[derive(Clone)]
#[cfg_attr(not(feature = "stat-latency"), derive(Serialize, Deserialize))]
pub struct Histogram {
    #[cfg(feature = "stat-latency")]
    inner: hdrhistogram::Histogram<u64>,
}

pub struct Timer {
    #[cfg(feature = "stat-latency")]
    start: std::time::Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            #[cfg(feature = "stat-latency")]
            start: std::time::Instant::now(),
        }
    }
}

impl Histogram {
    #[cfg(not(feature = "stat-latency"))]
    pub fn record(&mut self, _timer: Timer) {}

    #[cfg(feature = "stat-latency")]
    pub fn record(&mut self, timer: Timer) {
        let elapsed = timer.start.elapsed();
        self.inner.record(elapsed.as_nanos() as u64).unwrap();
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            #[cfg(feature = "stat-latency")]
            inner: hdrhistogram::Histogram::new(3).unwrap(),
        }
    }
}

#[cfg(feature = "stat-latency")]
impl serde::Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use hdrhistogram::serialization::Serializer as _;
        use hdrhistogram::serialization::V2DeflateSerializer;
        use serde::ser::Error as _;

        let mut buffer = Vec::new();

        {
            let mut encoder = base64::write::EncoderWriter::new(
                &mut buffer,
                &base64::engine::general_purpose::STANDARD,
            );

            V2DeflateSerializer::new()
                .serialize(&self.inner, &mut encoder)
                .map_err(S::Error::custom)?;
        }

        serializer.serialize_str(str::from_utf8(&buffer).map_err(S::Error::custom)?)
    }
}

#[cfg(feature = "stat-latency")]
impl<'de> serde::Deserialize<'de> for Histogram {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use hdrhistogram::serialization::Deserializer;
        use serde::de::Error as _;

        let mut string = <&'de str>::deserialize(deserializer).map(std::io::Cursor::new)?;
        let mut decoder = base64::read::DecoderReader::new(
            &mut string,
            &base64::engine::general_purpose::STANDARD,
        );

        Ok(Histogram {
            inner: Deserializer::new()
                .deserialize(&mut decoder)
                .map_err(D::Error::custom)?,
        })
    }
}
