mod auth;
mod components;
mod models;
mod pages;
mod router;
mod services;

use components::app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
