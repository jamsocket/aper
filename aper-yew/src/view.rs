use yew::Html;

trait View<T> {
    fn view(value: &T) -> Html;
}