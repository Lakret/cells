use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::btn::*;
use crate::modal::*;

#[derive(PartialEq, Properties)]
pub struct PasteModalProps {
  pub onpaste: Callback<String>,
  pub is_visible: bool,
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

  html! {
    <Modal title="Paste All Cells from JSON" is_visible={props.is_visible} onclose={props.onclose.clone()}>
      <div class="flex flex-col gap-4">
        <textarea
          cols="40"
          rows="5"
          placeholder="Paste cells JSON here and press 'Paste'"
          class="outline-none p-1 bg-violet-700 rounded-md"
          value={ (*value).clone() }
          {oninput}
        />

        <Btn
          title="Paste"
          color={ BtnColors::Green }
          onclick={ onpasteclick }/>
      </div>
    </Modal>
  }
}
