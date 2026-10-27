use anyhow::{bail, Result};

use aispy::{
    device::{DeviceEvent, DeviceEventKind},
    event::{
        ClockDomain,
        Event,
        EventKind,
        EventSource,
    },
};

pub fn normalize_device_event(raw: DeviceEvent) -> Result<Event> {
    let kind = match raw.kind {
        value if value == DeviceEventKind::Marker as u32 => {
            EventKind::DeviceMarker
        }

        value if value == DeviceEventKind::RangeBegin as u32 => {
            EventKind::DeviceRangeBegin
        }

        value if value == DeviceEventKind::RangeEnd as u32 => {
            EventKind::DeviceRangeEnd
        }

        other => {
            bail!("unknown device event kind {other}");
        }
    };

    Ok(Event {
        timestamp: raw.timestamp,
        clock_domain: ClockDomain::Device,

        source: EventSource::Device,
        kind,

        instance_id: Some(raw.instance_id),

        value_i64: Some(raw.payload as i64),

        ..Event::default()
    })
}
