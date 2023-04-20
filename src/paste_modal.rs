use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::btn::*;

#[derive(PartialEq, Properties)]
pub struct PasteModalProps {
  pub is_visible: bool,
  pub onpaste: Callback<String>,
  pub onclose: Callback<()>,
}

#[function_component]
pub fn PasteModal(props: &PasteModalProps) -> Html {
  let value = use_state(|| String::new());

  let oninput = {
    let value = value.clone();

    Callback::from(move |ev: InputEvent| {
      let input: HtmlTextAreaElement = ev.target().unwrap().dyn_into().unwrap();
      let new_value = input.value();

      value.set(new_value);
    })
  };

  let onpasteclick = {
    let value = value.clone();
    let parent_onpaste = props.onpaste.clone();
    let parent_onclose = props.onclose.clone();

    Callback::from(move |_ev: MouseEvent| {
      let v = value.to_string();
      value.set(String::new());

      parent_onclose.emit(());
      parent_onpaste.emit(v);
    })
  };

  let onclose = {
    let parent_onclose = props.onclose.clone();

    Callback::from(move |_ev: MouseEvent| {
      parent_onclose.emit(());
    })
  };

  if props.is_visible {
    html! {
      <div class={classes!(vec![
          "z-[100] fixed top-0 left-0 right-0 w-full p-4 overflow-x-hidden overflow-y-auto h-full max-h-full",
          "flex flex-col items-center justify-center backdrop-blur-sm"
        ])}
      >
        <div class="flex flex-col w-[32rem] py-2 px-4 bg-violet-900 rounded-md">
          <div class="flex justify-between pb-2">
            <h1 class="italic text-neutral-200">{ "Paste All Cells from JSON" }</h1>
            <button onclick={onclose}>{ "⨉" }</button>
          </div>
          <div class="flex flex-col gap-2">
            <textarea
              cols="40"
              rows="5"
              placeholder="Paste cells JSON here and press 'Paste'"
              class="outline-none p-1 bg-violet-700"
              value={ (*value).clone() }
              {oninput}
            />

            <Btn
              title="Paste"
              color={ BtnColors::Green }
              onclick={ onpasteclick }/>
          </div>
        </div>
      </div>
    }
  } else {
    html! {}
  }
}
