#![recursion_limit = "512"]

use yew::prelude::*;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::{components::RouterAnchor, prelude::*, switch::Permissive};

use pages::{about::About, game::Game, page_not_found::PageNotFound, register::Register};

mod models;
mod pages;
mod rest_helper;

#[derive(Clone, Debug, Switch)]
pub enum AppRoute {
    #[to = "/about/"]
    About,
    #[to = "/game/"]
    Game,
    #[to = "/page-not-found"]
    PageNotFound(Permissive<String>),
    #[to = "/"]
    Register,
}
pub type AppRouter = Router<AppRoute>;
pub type AppAnchor = RouterAnchor<AppRoute>;

enum Msg {
    ToggleNavbar,
}

struct Model {
    link: ComponentLink<Self>,
    navbar_active: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            navbar_active: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleNavbar => {
                self.navbar_active = !self.navbar_active;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                { self.view_nav() }

                <main class="section is-primary">
                    <Router<AppRoute, ()>
                        render = Router::render(Self::switch)
                    />
                </main>
                <footer class="footer">
                    <div class="content has-text-centered">
                        { "Powered by " }
                        <a href="https://rust-lang.org">{ "Rust" }</a>
                        { ", " }
                        <a href="https://yew.rs">{ "Yew" }</a>
                        { " and " }
                        <a href="https://bulma.io">{ "Bulma" }</a>
                    </div>
                </footer>
            </>
        }
    }
}
impl Model {
    fn view_nav(&self) -> Html {
        let Self {
            ref link,
            navbar_active,
            ..
        } = *self;
        let active_class = if navbar_active { "is-active" } else { "" };
        html! {
            <nav class="navbar is-primary" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <h1 class="navbar-item is-size-3">{ "Full-stack Connect5 in Rust" }</h1>

                    <a role="button"
                        class = classes!("navbar-burger", "burger", active_class)
                        aria-label="menu" aria-expanded="false"
                        onclick=link.callback(|_| Msg::ToggleNavbar)
                    >
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>
                <div class=classes!("navbar-menu", active_class)>
                    <div class="navbar-start">
                        <AppAnchor classes="navbar-item" route=AppRoute::Register>
                            { "NewGame" }
                        </AppAnchor>
                        <AppAnchor classes="navbar-item" route=AppRoute::Game>
                            { "Game" }
                        </AppAnchor>
                        <AppAnchor classes="navbar-item" route=AppRoute::About>
                            { "About" }
                        </AppAnchor>
                    </div>
                </div>
            </nav>
        }
    }
    fn switch(route: AppRoute) -> Html {
        match route {
            AppRoute::About => {
                html! { <About /> }
            }
            AppRoute::Register => {
                html! { <Register /> }
            }
            AppRoute::Game => {
                html! { <Game /> }
            }
            AppRoute::PageNotFound(Permissive(route)) => {
                html! { <PageNotFound route=route /> }
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::start_app::<Model>();
}
