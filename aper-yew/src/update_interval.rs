use std::time::Duration;
use yew::prelude::*;
use yew::services::interval::{IntervalService, IntervalTask};

#[derive(Properties, Clone)]
pub struct Props {
    pub callback: Callback<()>,
    pub interval_ms: u64,
}

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
