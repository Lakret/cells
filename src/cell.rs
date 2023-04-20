use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{cell_id::CellId, expr::Expr};

#[derive(PartialEq, Properties)]
pub struct CellProps {
  pub cell_id: CellId,
  pub input: Option<String>,
  pub expr: Option<Expr>,
  pub computed: Option<Expr>,
  pub onfocus: Callback<FocusEvent>,
  pub onfocusout: Callback<FocusEvent>,
  pub oninput: Callback<InputEvent>,
  // sending a custom string as if it was inputted into cell - useful for processing of keyboard input
  // on a focused cell, for example
  pub sendinput: Callback<String>,
}

/**
A cell that can be both selected and typed into.
*/
#[function_component]
pub fn Cell(props: &CellProps) -> Html {
  let div_select_mode = use_state(|| false);
  let input_mode = use_state(|| false);
  let input_ref = use_node_ref();

  let input_value = props.input.clone().unwrap_or_default();

  // if `computed_value` is present, show it in the div cell, otherwise show `value`
  let div_value = match props.computed {
    Some(Expr::Num(n)) => n.to_string(),
    _ => props.input.clone().unwrap_or_default(),
  };

  let onclick = {
    let div_select_mode = div_select_mode.clone();

    Callback::from(move |_ev: MouseEvent| {
      div_select_mode.set(true);
    })
  };

  let ondblclick = {
    let input_mode = input_mode.clone();
    let input_ref = input_ref.clone();

    Callback::from(move |_ev: MouseEvent| {
      input_mode.set(true);

      input_ref
        .cast::<HtmlInputElement>()
        .expect("ref is not attached to an input")
        .focus()
        .expect("cannot focus");
    })
  };

  let onkeypress = {
    let input_mode = input_mode.clone();
    let input_ref = input_ref.clone();

    let parent_sendinput = props.sendinput.clone();

    Callback::from(move |ev: KeyboardEvent| {
      input_mode.set(true);

      input_ref
        .cast::<HtmlInputElement>()
        .expect("ref is not attached to an input")
        .focus()
        .expect("cannot focus");

      parent_sendinput.emit(ev.key());
    })
  };

  let div_onfocusout = {
    let div_select_mode = div_select_mode.clone();
    let parent_onfocusout = props.onfocusout.clone();

    Callback::from(move |ev: FocusEvent| {
      div_select_mode.set(false);
      parent_onfocusout.emit(ev);
    })
  };

  let input_onfocusout = {
    let input_mode = input_mode.clone();
    let parent_onfocusout = props.onfocusout.clone();

    Callback::from(move |ev: FocusEvent| {
      input_mode.set(false);
      parent_onfocusout.emit(ev);
    })
  };

  // note that the div gets a tabindex to allow focus & keyboard events;
  // `input_ref` is used to focus the input
  html! {
    <td>
      <div class="flex">
        <input
          ref={ input_ref }
          id={ props.cell_id.to_string() }
          type="text"
          class={classes!(vec![
            "px-2 py-0.5 w-[16rem] h-[2.125rem] outline-none text-right snap-start",
            "border-collapse border-[1px] border-indigo-900 bg-indigo-800 font-mono",
            if *input_mode { "z-10" } else { "z-0 select-none" }
          ])}
          value={ input_value }
          onfocus={ props.onfocus.clone() }
          oninput={ props.oninput.clone() }
          onfocusout={ input_onfocusout  }
        />

        <div
          id={ props.cell_id.to_string() }
          tabindex="0"
          class={classes!(vec![
            "flex px-2 py-0.5 w-[16rem] -ml-[16rem] h-[2.125rem]",
            "border-[1px] border-indigo-900",
            if *input_mode { "z-0" } else { "z-10" },
            if *div_select_mode { "bg-indigo-700" } else { "bg-indigo-800" },
          ])}
          {onclick}
          {ondblclick}
          {onkeypress}
          onfocusout={ div_onfocusout }
        >
          <span class="grow text-right select-none font-mono">{ div_value }</span>
        </div>
      </div>
    </td>
  }
}
