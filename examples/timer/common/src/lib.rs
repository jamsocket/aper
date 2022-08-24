use aper::{NeverConflict, StateMachine};
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

impl Default for Timer {
    fn default() -> Self {
        Timer {
            value: 0,
            last_increment: Utc::now(),
        }
    }
}

impl StateMachine for Timer {
    type Transition = TransitionEvent<TimerEvent>;
    type Conflict = NeverConflict;

    fn apply(&self, event: &Self::Transition) -> Result<Self, NeverConflict> {
        let mut new_self = self.clone();
        match event.transition {
            TimerEvent::Reset => new_self.value = 0,
            TimerEvent::Increment => {
                new_self.value += 1;
                new_self.last_increment = event.timestamp;
            }
        }

        Ok(new_self)
    }
}

impl StateProgram for Timer {
    type T = TimerEvent;

    fn new() -> Self {
        Timer::default()
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
