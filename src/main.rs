// use crate::events;
use crate::events::{CombinedEvent, Event};
use std::fmt::Display;
use std::{thread, time, time::Duration};

pub mod events;

const SAMPLE_RATE: u64 = 100;
const SAMPLE_MODULO: u64 = 1000 / SAMPLE_RATE;

struct Timeline {
    events: Vec<events::Event>,
}

impl Timeline {
    fn add_event(&mut self, start_time: u32, duration: u32) {
        let next_id = self.events.last().map_or(0, |e| e.id + 1);

        // Validate that start time is greater than or equal to last event time
        self.events.push(Event {
            id: next_id,
            start_time,
            duration,
        });
    }
}

fn main() {
    let mut timeline = Timeline { events: Vec::new() };
    timeline.add_event(0, 500);
    timeline.add_event(1000, 1500);
    timeline.add_event(2000, 100);
    timeline.add_event(2000, 3000);
    timeline.add_event(3000, 10);

    let start_time = time::SystemTime::now();
    let mut events_iter = timeline.events.iter();
    let mut next_event = events_iter.next();

    let mut combined_event = CombinedEvent::new();

    loop {
        let now = time::SystemTime::now();
        let delta = now.duration_since(start_time).unwrap();

        if next_event.is_some() && delta.as_millis() as u32 > next_event.unwrap().start_time {
            println!(
                "Triggering event {}, expected: {}, actual: {}",
                next_event.unwrap().id,
                next_event.unwrap().start_time,
                delta.as_millis()
            );
            combined_event.add_event(next_event.unwrap());
            next_event = events_iter.next();
        }

        combined_event.tick(&delta);

        if next_event.is_none() && combined_event.is_empty() {
            break;
        }

        if delta.as_millis() as u64 % SAMPLE_MODULO == 0 {
            println!(
                "sample value at {} is {}",
                delta.as_millis(),
                combined_event.value()
            );
        }
    }
}
