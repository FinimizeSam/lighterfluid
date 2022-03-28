#![allow(unused)]
#![allow(incomplete_features)]
#![deny(unused_must_use)]
#![feature(async_stream)]
#![feature(const_option)]
#![feature(inline_const_pat)]
#![feature(is_some_with)]
#![feature(once_cell)]
#![feature(slice_pattern)]
#![feature(stream_from_iter)]
#![feature(type_alias_impl_trait)]
#![feature(unsized_locals)]

use std::ffi::OsString;
use std::stream::Stream;

mod events;

fn main() -> Result<(), std::io::Error> {
    let input_path: OsString = std::env::var_os("INPUT_PATH").unwrap_or("./data.csv".into());

    /// Stream events
    let in_stream = events::load_events(input_path)?;

    Ok(())
}

pub struct Event {
    event: EventType,
    user_id: String,
    post_id: String,
    time: u32,
}

#[derive(Debug)]
enum EventType {
    Started,
    Finished,
}
