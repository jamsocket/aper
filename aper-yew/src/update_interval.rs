use std::time::Duration;
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};

/// Props of [UpdateInterval].
#[derive(Properties, Clone)]
pub struct Props {
    /// The [yew::Callback] which gets called at the interval.
    pub callback: Callback<()>,

    /// The number of milliseconds to wait between calls to `callback`.
    pub interval_ms: u64,
}

/// A Yew component that calls the given callback at a regular interval.
/// It is a useful way to automatically refresh a [crate::View], since the
/// state view itself cannot own an [IntervalTask].
pub struct UpdateInterval {
    props: Props,

    #[allow(unused)]
    interval_task: IntervalTask,
}

impl Component for UpdateInterval {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let interval_task = IntervalService::spawn(
            Duration::from_millis(props.interval_ms),
            props.callback.clone(),
        );
        Self {
            props,
            interval_task,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }

    fn view(&self) -> Html {
        return html! {};
    }
}
