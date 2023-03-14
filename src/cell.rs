// TODO: kill once the cells work properly in the table

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::cell_id::CellId;

#[derive(Properties, PartialEq)]
pub struct CellProps {
    pub cell_id: CellId,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub class: AttrValue,
}

#[function_component]
pub fn Cell(props: &CellProps) -> Html {
    let cell_val = use_state(|| String::new());

    let on_change = props.on_change.clone();
    let onkeyup = {
        let cell_val = cell_val.clone();
        move |ev: InputEvent| {
            let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
            let val = input.value();
            cell_val.set(val.clone());
            on_change.emit(val);
        }
    };

    let cell_class = "px-2 py-0.5 w-[10rem] outline-none text-right snap-start
        border-collapse border-[1px] border-indigo-900 bg-indigo-800";

    html! {
        <td>
            <input
                type="text"
                id={ props.cell_id.to_string() }
                value={ (*cell_val).clone() }
                oninput={ onkeyup.clone() }
                class={ format!("{} {}", props.class, cell_class)}
            />
        </td>
    }
}
