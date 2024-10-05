use aper::{data_structures::atom::Atom, Aper, AperSync, IntentMetadata};
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

    fn apply(&mut self, intent: &Self::Intent, metadata: &IntentMetadata) -> Result<(), ()> {
        match intent {
            TimerIntent::Reset => self.value.set(0),
            TimerIntent::Increment => {
                self.value.set(self.value.get() + 1);
                self.last_increment.set(metadata.timestamp);
            }
        }

        Ok(())
    }

    fn suspended_event(&self) -> Option<(DateTime<Utc>, TimerIntent)> {
        let next_event = self
            .last_increment
            .get()
            .checked_add_signed(Duration::seconds(1))
            .unwrap();

        Some((next_event, TimerIntent::Increment))
    }
}
