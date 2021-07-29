use aper::{PlayerID, StateProgram, StateProgramFactory, StateUpdateMessage, TransitionEvent};
use chrono::Utc;
use jamsocket::{JamsocketContext, JamsocketService, JamsocketServiceBuilder, MessageRecipient};
use std::marker::PhantomData;

pub struct AperJamsocketService<P: StateProgram, C: JamsocketContext> {
    state: P,
    suspended_event: Option<TransitionEvent<P::T>>,
    context: C,
}

impl<P: StateProgram, C: JamsocketContext> AperJamsocketService<P, C> {
    fn update_suspended_event(&mut self) {
        let susp = self.state.suspended_event();
        if susp == self.suspended_event {
            return;
        }

        if let Some(ev) = &susp {
            if let Ok(dur) = ev.timestamp.signed_duration_since(Utc::now()).to_std() {
                self.context.set_timer(dur.as_millis() as u32);
            }
        }

        self.suspended_event = susp;
    }

    fn process_transition(&mut self, user: u32, transition: TransitionEvent<P::T>) {
        if transition.player != Some(PlayerID(user as usize)) {
            log::warn!(
                "Received a transition from a client with an invalid player ID. {:?} != {}",
                transition.player,
                user
            );
            return;
        }
        self.state.apply(transition.clone());
        self.context.send_message(
            MessageRecipient::Broadcast,
            serde_json::to_string(&StateUpdateMessage::TransitionState::<P>(transition))
                .unwrap()
                .as_str(),
        );
        self.update_suspended_event();
    }
}

impl<P: StateProgram, C: JamsocketContext> JamsocketService for AperJamsocketService<P, C> {
    fn connect(&mut self, user: u32) {
        self.context.send_message(
            MessageRecipient::User(user),
            serde_json::to_string(&StateUpdateMessage::ReplaceState::<P>(
                self.state.clone(),
                Utc::now(),
                PlayerID(user as usize),
            ))
            .unwrap()
            .as_str(),
        );
    }

    fn disconnect(&mut self, _user: u32) {}

    fn message(&mut self, user: u32, message: &str) {
        let transition: TransitionEvent<P::T> = serde_json::from_str(message).unwrap();
        self.process_transition(user, transition);
    }

    fn binary(&mut self, user: u32, message: &[u8]) {
        let transition: TransitionEvent<P::T> = bincode::deserialize(message).unwrap();
        self.process_transition(user, transition);
    }

    fn timer(&mut self) {
        if let Some(event) = self.suspended_event.take() {
            self.state.apply(event);
            self.update_suspended_event();
        }
    }
}

#[derive(Clone)]
pub struct AperJamsocketServiceBuilder<
    K: StateProgramFactory + Send + Sync + Clone,
    C: JamsocketContext + Unpin + 'static,
> {
    ph_c: PhantomData<C>,

    state_program_factory: K,
}

impl<K: StateProgramFactory + Send + Sync + Clone, C: JamsocketContext + Unpin + 'static>
    AperJamsocketServiceBuilder<K, C>
{
    pub fn new(state_program_factory: K) -> Self {
        AperJamsocketServiceBuilder {
            ph_c: Default::default(),
            state_program_factory,
        }
    }
}

impl<K: StateProgramFactory + Send + Sync + Clone, C: JamsocketContext + Unpin + 'static>
    JamsocketServiceBuilder<C> for AperJamsocketServiceBuilder<K, C>
{
    type Service = AperJamsocketService<K::State, C>;

    fn build(mut self, _room_id: &str, context: C) -> Self::Service {
        let state = self.state_program_factory.create();

        AperJamsocketService {
            state,
            suspended_event: None,
            context,
        }
    }
}
