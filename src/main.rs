use yew::prelude::*;

mod cell_id;
mod expr;
mod table;
use table::Table;

// TODO: the whole hooks / callbacks thing becomes unwieldy here => use normal TEA
// TODO: selected cell, highlight row and col names on cell selection
#[function_component]
fn App() -> Html {
    html! {
        <Table />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
