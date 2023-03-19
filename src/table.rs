use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::cell_id::CellId;

#[derive(Debug, PartialEq)]
pub enum Msg {
  CellFocused { cell_id: CellId, value: String },
  CellLostFocus { cell_id: CellId },
  CellChanged { cell_id: CellId, new_value: String },
}

#[derive(Default)]
pub struct Table {
  big_input_text: String,
  focused_cell: Option<CellId>,
  values: HashMap<CellId, String>,
  computed_values: HashMap<CellId, String>,
}

// TODO: cell reference insertion mode when cell is edited and starts with =

impl Component for Table {
  type Message = Msg;
  type Properties = ();

  fn create(_ctx: &Context<Self>) -> Self {
    Table::default()
  }

  fn view(&self, ctx: &Context<Self>) -> Html {
    html! {
      <div class="mx-auto flex flex-col h-full max-h-full w-full max-w-full text-white text-xl grow-0">
        <div class="w-screen grow-0 sticky top-0 left-0 z-50 flex gap-4 px-4 py-4 bg-indigo-900">
          <input
            type="text"
            value={ self.big_input_text.clone() }
            class={classes!(vec![
              "grow ml-[3rem] px-2 py-0.5 outline-none font-mono",
              "border-[1px] border-indigo-900 bg-indigo-800"
            ])}
          />

          <button class={classes!(vec![
            "flex flex-col items-center px-4 py-0.5 cursor-pointer rounded-md",
            "bg-purple-800 hover:bg-purple-700"
          ])}>
            { "Clear All" }
          </button>

          <button class={classes!(vec![
            "flex flex-col items-center px-4 py-0.5 cursor-pointer rounded-md",
            "bg-green-800 hover:bg-green-700"
          ])}>
            { "Help" }
          </button>
        </div>

          <div class="overflow-scroll snap-y snap-mandatory pb-4">
            <table class="table table-fixed">
              <thead>
                <tr class="snap-start">
                  <th class="sticky top-0 left-0 snap-start pl-6 pr-4 z-40 w-full bg-indigo-900">
                  </th>
                  {
                    // col id headers
                    ('A'..='Z').map(move |col| {
                      let header_style =
                          match self.focused_cell {
                              Some(CellId{ col: focused_col, .. }) if focused_col == col =>
                                  "text-neutral-300 hover:text-neutral-200",
                              _ => "text-neutral-400 hover:text-neutral-300",
                          };

                      html! {
                        <th id={ format!("header-col-{col}") }
                          class={classes!(vec![
                              "z-30 sticky top-0 snap-start bg-clip-padding bg-indigo-900",
                              "text-center",
                              header_style
                          ])}>
                          { col }
                        </th>
                      }
                    }).collect::<Html>()
                  }
                </tr>
              </thead>
              <tbody>
                {
                  (1..=50).map(move |row| {
                    html! {
                      <tr>
                      {
                        ('@'..='Z').map(move |col| {
                          // row id header
                          if col == '@' {
                            let header_style =
                              match self.focused_cell {
                                Some(CellId{ row: focused_row, .. }) if focused_row == row =>
                                  "text-neutral-300 hover:text-neutral-200",
                                _ => "text-neutral-400 hover:text-neutral-300",
                              };

                            html! {
                              <th id={ format!("header-row-{row}") }
                                class={
                                classes!(vec![
                                    "z-[35] sticky left-0 snap-start pl-6 pr-4 bg-indigo-900 text-right",
                                    header_style
                                ])
                              }>
                                  { row }
                              </th>
                            }
                          } else {
                            let cell_id = CellId { col, row };
                            html! {
                              <Cell
                                {cell_id}
                                value={ self.values.get(&cell_id).map(|v| v.clone()) }
                                computed_value={ self.computed_values.get(&cell_id).map(|v| v.clone()) }
                                onfocus={
                                  ctx.link().callback(move |ev: FocusEvent| {
                                    let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                    let value = input.value();

                                    Msg::CellFocused { cell_id, value }
                                  })
                                }
                                onfocusout={
                                  ctx.link().callback(move |ev: FocusEvent| {
                                    Msg::CellLostFocus { cell_id }
                                  })
                                }
                                oninput={
                                  ctx.link().callback(move |ev: InputEvent| {
                                    let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                    let new_value = input.value();

                                    Msg::CellChanged { cell_id, new_value }
                                  })
                                }
                              />
                            }
                          }
                        }).collect::<Html>()
                      }
                      </tr>
                    }
                  }).collect::<Html>()
                }
              </tbody>
            </table>
          </div>
      </div>
    }
  }

  fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      Msg::CellFocused { cell_id, value } => {
        self.focused_cell = Some(cell_id);
        self.big_input_text = value;
        true
      }
      Msg::CellLostFocus { .. } => {
        self.focused_cell = None;
        self.big_input_text = String::from("");
        true
      }
      Msg::CellChanged { cell_id, new_value } => {
        self.values.insert(cell_id, new_value.clone());
        // TODO: actual computation
        // self
        //   .computed_values
        //   .insert(cell_id, format!("COMP{}", new_value));
        self.big_input_text = new_value;
        true
      }
    }
  }
}

#[derive(PartialEq, Properties)]
pub struct CellProps {
  pub cell_id: CellId,
  pub value: Option<String>,
  pub computed_value: Option<String>,
  pub onfocus: Callback<FocusEvent>,
  pub onfocusout: Callback<FocusEvent>,
  pub oninput: Callback<InputEvent>,
}

/**
A cell that can be both selected and typed into.
*/
#[function_component]
fn Cell(props: &CellProps) -> Html {
  let div_select_mode = use_state(|| false);
  let input_mode = use_state(|| false);
  let input_ref = use_node_ref();

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

  // if `computed_value` is present, show it in the div cell, otherwise show `value`
  let div_value = props.computed_value.clone().or(props.value.clone());

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
            "px-2 py-0.5 w-[10rem] h-[2.125rem] outline-none text-right snap-start",
            "border-collapse border-[1px] border-indigo-900 bg-indigo-800 font-mono",
            if *input_mode { "z-10" } else { "z-0 select-none" }
          ])}
          value={ props.value.clone() }
          onfocus={ props.onfocus.clone() }
          oninput={ props.oninput.clone() }
          onfocusout={ input_onfocusout  }
        />
        <div
          id={ props.cell_id.to_string() }
          tabindex="0"
          class={classes!(vec![
            "flex px-2 py-0.5 w-[10rem] -ml-[10rem] h-[2.125rem]",
            "border-[1px] border-indigo-900",
            if *input_mode { "z-0" } else { "z-10" },
            if *div_select_mode { "bg-indigo-700" } else { "bg-indigo-800" },
          ])}
          {onclick}
          {ondblclick}
          onfocusout={ div_onfocusout }
        >
          {
            match &div_value {
              None => {
                html!{}
              },
              Some(value) => {
                html!{
                  <span class="grow text-right select-none font-mono">{ value }</span>
                }
              }
            }
          }
        </div>
      </div>
    </td>
  }
}
