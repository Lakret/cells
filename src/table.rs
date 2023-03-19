use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, HtmlInputElement, Window};
use yew::prelude::*;
use yew::props;

use crate::cell_id::CellId;

#[derive(Debug, PartialEq)]
pub enum Msg {
  CellFocused { cell_id: CellId, value: String },
  CellChanged { cell_id: CellId, new_value: String },
}

#[derive(Default)]
pub struct Table {
  big_input_text: String,
  focused_cell: Option<CellId>,
}

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

          <button class="px-4 py-0.5 cursor-pointer rounded-md bg-purple-800 hover:bg-purple-700">
            { "Clear All" }
          </button>

          <button class="px-4 py-0.5 cursor-pointer rounded-md bg-green-800 hover:bg-green-700">
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
                                onfocus={
                                  ctx.link().callback(move |ev: FocusEvent| {
                                    let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                    let value = input.value();

                                    Msg::CellFocused { cell_id, value }
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
      Msg::CellChanged { new_value, .. } => {
        self.big_input_text = new_value;
        true
      }
    }
  }
}

// TODO: function component for Cell and make them selectable / make into inputs on double click

#[derive(PartialEq, Properties)]
pub struct CellProps {
  pub cell_id: CellId,
  pub onfocus: Callback<FocusEvent>,
  pub oninput: Callback<InputEvent>,
}

/**
A cell that can be both selected and typed into.
*/
#[function_component]
fn Cell(props: &CellProps) -> Html {
  let input_mode = use_state(|| false);
  let input_ref = use_node_ref();

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

  html! {
    <td>
      <div class="flex">
        <input
          ref={input_ref}
          id={ props.cell_id.to_string() }
          type="text"
          class={classes!(vec![
            "px-2 py-0.5 w-[10rem] h-[2.125rem] outline-none text-right snap-start",
            "border-collapse border-[1px] border-indigo-900 bg-indigo-800 font-mono",
            if *input_mode { "z-10" } else { "z-0" }
          ])}
          onfocus={ props.onfocus.clone() }
          oninput={ props.oninput.clone() }
        />
        <div
          id={ props.cell_id.to_string() }
          type="text"
          class={classes!(vec![
            "px-2 py-0.5 w-[10rem] -ml-[10rem] h-[2.125rem]",
            "border-[1px] border-indigo-900 bg-indigo-800",
            if *input_mode { "z-0" } else { "z-10" }
          ])}
          {ondblclick}
        />
      </div>
    </td>
  }
}
