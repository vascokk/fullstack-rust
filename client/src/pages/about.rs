use yew::prelude::*;

pub struct About;
impl Component for About {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="tile is-ancestor is-vertical">
                <div class="tile is-child hero">
                    <div class="hero-body container pb-0">
                        <h1 class="title is-1">{ "About" }</h1>
                        <h2 class="subtitle">{ "Full-stack Rust with WebAssembly implementation of the Connect5 game" }</h2>
                    </div>
                </div>
            </div>
        }
    }
}
