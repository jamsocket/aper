use aper::{StateMachine, NeverConflict};
use aper_stateroom::{StateProgram, TransitionEvent};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timer {
    pub value: i64,
    pub last_increment: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TimerEvent {
    Reset,
    Increment,
}

impl StateMachine for Timer {
    type Transition = TransitionEvent<TimerEvent>;
    type Conflict = NeverConflict;

    fn apply(&mut self, event: Self::Transition) -> Result<(), NeverConflict> {
        match event.transition {
            TimerEvent::Reset => self.value = 0,
            TimerEvent::Increment => {
                self.value += 1;
                self.last_increment = event.timestamp;
            }
        }

        Ok(())
    }
}

impl StateProgram for Timer {
    type T = TimerEvent;

    fn new(_: &str) -> Self {
        Timer {
            value: 0,
            last_increment: Utc::now(),
        }
    }

    fn suspended_event(&self) -> Option<TransitionEvent<Self::T>> {
        let next_event = self
            .last_increment
            .checked_add_signed(Duration::seconds(1))
            .unwrap();

        Some(TransitionEvent::new(
            None,
            next_event,
            TimerEvent::Increment,
        ))
    }
}
