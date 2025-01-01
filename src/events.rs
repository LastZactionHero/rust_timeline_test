use std::time::Duration;

#[derive(Debug)]
pub struct Event {
    pub id: u32,
    pub start_time: u32,
    pub duration: u32,
}

impl Event {
    pub fn end_time(&self) -> u32 {
        self.start_time + self.duration
    }

    pub fn value(&self) -> u32 {
        self.id
    }
}

pub struct CombinedEvent<'a> {
    pub events: Vec<&'a Event>,
}

impl<'a> CombinedEvent<'a> {
    pub fn new() -> CombinedEvent<'a> {
        CombinedEvent { events: Vec::new() }
    }

    pub fn value(&self) -> u32 {
        self.events.len() as u32
    }

    pub fn add_event(&mut self, event: &'a Event) {
        self.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn tick(&mut self, delta: &Duration) {
        for active_event in &self.events {
            if active_event.end_time() <= delta.as_millis() as u32 {
                println!(
                    "Event done! {} expected {} actual {}",
                    active_event.id,
                    active_event.end_time(),
                    delta.as_millis()
                );
            }
        }
        self.events = self
            .events
            .clone()
            .into_iter()
            .filter(|event| event.end_time() > delta.as_millis() as u32)
            .collect();
    }
}
