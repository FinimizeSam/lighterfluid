use std::ffi::OsString;
use std::io::Error;
use std::lazy::SyncLazy;
use std::stream::Stream;

use regex::Regex;
use serde::Deserialize;

use super::{Event, EventType};

/// Types

#[derive(Debug, Deserialize)]
struct RawEvent {
    #[serde(rename = "event")]
    event_type: String,
    properties: RawEventProperties,
}

#[derive(Debug, Deserialize)]
struct RawEventProperties {
    #[serde(rename = "id")]
    post_id: String,
    #[serde(rename = "time")]
    time: u32,

    /// Both of the below are user email fields. They are ordered
    /// by parsing priority: e.g. it's possible that field B will
    /// contain the wrong kind of value if field A already has an
    /// email address for its value.
    #[serde(rename = "$distinct_id_before_identity")]
    distinct_id_before_identity: Option<String>,
    #[serde(rename = "$user_id")]
    user_id: Option<String>,
    #[serde(rename = "distinct_id")]
    distinct_id: Option<String>,
}

/// Consts

const EMAIL_RE: SyncLazy<Regex> = SyncLazy::new(|| {
    Regex::new("[A-Za-z0-9.-_]+@([A-Za-z0-9-]+.){1,3}([A-Za-z0-9-]+)(.[A-Za-z0-9-]+){0,2}")
        .expect("failed to compile regexp")
});

/// Functions

/// Returns an iterator over all valid events in a given file.
pub fn load_events(fname: OsString) -> Result<impl Stream<Item = Event>, Error> {
    use std::fs::File;
    use std::io::BufReader;

    use serde_json::{Deserializer, Value};

    // Open the file in read-only mode with buffer.
    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an iterator of `Event`s.
    let disk_iter = Deserializer::from_reader(reader)
        .into_iter::<Value>()
        .filter_map(Result::ok)
        .map(serde_json::from_value::<RawEvent>)
        .filter_map(|opt| {
            opt.or_else(|e| {
                eprintln!("deserialization error: {}", e);
                Err(e)
            })
            .ok()
        })
        .map(|raw| Event {
            user_id: raw
                .properties
                .user_id
                .or(raw.properties.distinct_id_before_identity)
                .or(raw.properties.distinct_id)
                .expect("no user id found"),
            event: match &*raw.event_type {
                "Started Content Piece" => EventType::Started,
                "End Read Content" => EventType::Finished,
                un => unimplemented!("unimplemented event type: {}", un),
            },
            post_id: raw.properties.post_id,
            time: raw.properties.time,
        })
        .filter(|event| match EMAIL_RE.is_match(&event.user_id) {
            true => true,
            false => {
                if option_env!("DEBUG").is_some_with(|&e| e == "true") {
                    eprintln!("[DATA] [REJECT EMAIL] '{}'", &event.user_id);
                }
                false
            }
        });

    let stream = std::stream::from_iter(disk_iter);

    Ok(stream)
}
