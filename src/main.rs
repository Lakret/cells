use yew::prelude::*;

mod cell_id;
mod expr;
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
