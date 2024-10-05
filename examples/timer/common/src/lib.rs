use aper::{data_structures::atom::Atom, Aper, AperSync, IntentEvent};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(AperSync, Clone)]
pub struct Timer {
    pub value: Atom<i64>,
    pub last_increment: Atom<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TimerIntent {
    Reset,
    Increment,
}

impl Aper for Timer {
    type Intent = TimerIntent;
    type Error = ();

    fn apply(&mut self, event: &IntentEvent<Self::Intent>) -> Result<(), ()> {
        match event.intent {
            TimerIntent::Reset => self.value.set(0),
            TimerIntent::Increment => {
                self.value.set(self.value.get() + 1);
                self.last_increment.set(event.timestamp);
            }
        }

        Ok(())
    }

    fn suspended_event(&self) -> Option<IntentEvent<TimerIntent>> {
        let next_event = self
            .last_increment
            .get()
            .checked_add_signed(Duration::seconds(1))
            .unwrap();

        Some(IntentEvent::new(None, next_event, TimerIntent::Increment))
    }
}
