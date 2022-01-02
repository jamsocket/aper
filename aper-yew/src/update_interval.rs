use gloo_timers::callback::Timeout;
use yew::prelude::*;

/// Props of [UpdateInterval].
#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    /// The [yew::Callback] which gets called at the interval.
    pub callback: Callback<()>,

    /// The number of milliseconds to wait between calls to `callback`.
    pub interval_ms: u32,
}

/// A Yew component that calls the given callback at a regular interval.
/// It is a useful way to automatically refresh a [crate::View], since the
/// state view itself cannot own an [IntervalTask].
pub struct UpdateInterval {
    #[allow(unused)]
    interval_task: Timeout,
}

impl Component for UpdateInterval {
    type Message = ();
    type Properties = Props;

    fn create(context: &yew::Context<Self>) -> Self {
        // let interval_task = IntervalService::spawn(
        //     Duration::from_millis(props.interval_ms),
        //     context.link().callback.clone(),
        // );

        let callback = context.props().callback.clone();
        let interval_task = Timeout::new(context.props().interval_ms, move || callback.emit(()));

        Self { interval_task }
    }

    fn view(&self, _context: &yew::Context<Self>) -> Html {
        return html! {};
    }
}
