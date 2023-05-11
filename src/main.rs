use yew::prelude::*;

use cells::table::Table;

#[function_component]
fn App() -> Html {
  html! {
      <Table />
  }
}

fn main() {
  yew::Renderer::<App>::new().render();
}
