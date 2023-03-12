use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let cell_val = use_state(|| String::new());
    let onkeyup = {
        let cell_val = cell_val.clone();
        move |ev: KeyboardEvent| {
            let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
            cell_val.set(input.value());
        }
    };

    html! {
        <div class="mx-auto container py-10 text-white text-xl grow-1">
            <div class="flex flex-row flex-wrap">
            {
                (1..=10).map(|_input_idx| html! {
                    <input type="text"
                        value={ (*cell_val).clone() }
                        onkeyup={onkeyup.clone()}
                        class="bg-indigo-800 outline-none px-2 py-0.5" />
                }).collect::<Html>()
            }
            </div>

            <p>{ (*cell_val).clone() }</p>

            <br/>
            <button class="rounded-md cursor-pointer" {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
