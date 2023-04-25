use yew::prelude::*;

#[macro_use]
extern crate lazy_static;

mod btn;
mod cell;
mod cell_id;
mod expr;
mod help_modal;
mod modal;
mod parser;
mod paste_modal;
mod table;
use table::Table;

#[function_component]
fn App() -> Html {
  html! {
      <Table />
  }
}

fn main() {
  yew::Renderer::<App>::new().render();
}
