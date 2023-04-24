use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::*;
use web_sys::console::log_1;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::btn::*;
use crate::cell::*;
use crate::cell_id::CellId;
use crate::expr::{eval, Expr};
use crate::parser::parse;
use crate::paste_modal::PasteModal;

// TODO: enable cell reference insertion if a cell is input and starts with =
// TODO: keyboard navigation between cells
#[derive(Debug, PartialEq)]
pub enum Msg {
  CopyAll,
  PasteAll,
  PasteAllContent { serialized_table: String },
  PasteModalClose,
  Help,
  CellFocused { cell_id: CellId, value: String },
  CellLostFocus { cell_id: CellId },
  CellBecameInput { cell_id: CellId },
  CellLostInput { cell_id: CellId },
  CellChanged { cell_id: CellId, new_value: String },
  BigInputFocused,
  BigInputChanged { new_value: String },
}

#[derive(Default, Debug)]
pub struct Table {
  big_input_text: String,
  focused_cell: Option<CellId>,
  input_cell: Option<CellId>,
  prev_focused_cell: Option<CellId>,
  paste_modal_visible: bool,
  inputs: HashMap<CellId, String>,
  exprs: HashMap<CellId, Expr>,
  computed: HashMap<CellId, Expr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableTable {
  // serde-json doesn't allow using non-string keys in hashmaps
  inputs: HashMap<String, String>,
}

// TODO: cell reference insertion mode when cell is edited and starts with =

impl Table {
  fn reeval(&mut self) {
    match eval(&self.exprs) {
      Ok(computed) => self.computed = computed,
      Err(err) => log_1(&JsValue::from_str(&format!(
        "Failed when trying to recompute: {err}."
      ))),
    };
  }

  fn cells_to_str(&self) -> String {
    let t = SerializableTable {
      inputs: self
        .inputs
        .iter()
        .map(|(cell_id, input)| (cell_id.to_string(), input.clone()))
        .collect(),
    };
    serde_json::to_string(&t).unwrap()
  }

  fn cells_from_str(&mut self, encoded: &str) {
    match serde_json::from_str::<SerializableTable>(encoded) {
      Ok(serializable_table) => {
        let inputs = serializable_table
          .inputs
          .into_iter()
          .map(|(cell_id, input)| {
            CellId::try_from(cell_id.as_ref()).map(|cell_id| (cell_id, input))
          })
          .collect::<Result<HashMap<_, _>, _>>();

        match inputs {
          Ok(inputs) => {
            self.inputs = inputs;

            self.exprs = self
              .inputs
              .iter()
              .filter_map(|(cell_id, input)| match parse(input) {
                Ok(expr) => Some((cell_id.clone(), expr.clone())),
                Err(err) => {
                  log_1(&JsValue::from(format!(
                    "cannot parse `{}` due to: {err:?}",
                    input
                  )));
                  None
                }
              })
              .collect();

            self.reeval();
          }
          Err(err) => log_1(&JsValue::from(format!(
            "cannot deserialize table from pasted input due to: {err:?}"
          ))),
        }
      }
      Err(err) => log_1(&JsValue::from(format!(
        "failed when trying to deserialized table: {err:?}"
      ))),
    }
  }
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
        <PasteModal
          is_visible={ self.paste_modal_visible }
          onclose={ ctx.link().callback(move |()| { Msg::PasteModalClose })}
          onpaste={ ctx.link().callback(move |serialized_table: String| {
            Msg::PasteAllContent { serialized_table }
          })}
        />

        <div class="w-screen grow-0 sticky top-0 left-0 z-50 flex gap-4 px-4 py-4 bg-indigo-900">
          <input
            type="text"
            class={classes!(vec![
              "grow ml-[3rem] px-2 py-0.5 outline-none font-mono border-[1px] border-indigo-900 bg-indigo-800"
            ])}
            value={ self.big_input_text.clone() }
            onfocusin={ ctx.link().callback(move |_ev: FocusEvent| { Msg::BigInputFocused })}
            oninput={ ctx.link().callback(move |ev: InputEvent| {
              let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
              let new_value = input.value();

              Msg::BigInputChanged { new_value }
            }) }
          />

          <Btn
            title="Copy All"
            color={ BtnColors::Purple }
            onclick={ ctx.link().callback(move |_ev: MouseEvent| { Msg::CopyAll }) }
          />
          <Btn
            title="Paste All"
            color={ BtnColors::Violet }
            onclick={ ctx.link().callback(move |_ev: MouseEvent| { Msg::PasteAll }) }
          />
          <Btn
            title="Help"
            color={ BtnColors::Green }
            onclick={ ctx.link().callback(move |_ev: MouseEvent| { Msg::Help }) }
          />
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
                            "z-30 sticky top-0 snap-start bg-clip-padding bg-indigo-900 text-center",
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
                              is_focused={self.focused_cell == Some(cell_id)}
                              is_input={self.input_cell == Some(cell_id)}
                              input={self.inputs.get(&cell_id).map(|x| x.clone())}
                              expr={self.exprs.get(&cell_id).map(|x| x.clone())}
                              computed={self.computed.get(&cell_id).map(|x| x.clone())}
                              onfocused={
                                ctx.link().callback(move |(cell_id, value)| {
                                  // let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                  // let value = input.value();

                                  Msg::CellFocused { cell_id, value }
                                })
                              }
                              onfocusout={
                                ctx.link().callback(move |_ev: FocusEvent| {
                                  Msg::CellLostFocus { cell_id }
                                })
                              }
                              onbecameinput={
                                ctx.link().callback(move |cell_id| {
                                  Msg::CellBecameInput { cell_id }
                                })
                              }
                              onlostinput={
                                ctx.link().callback(move |cell_id| {
                                  Msg::CellLostInput { cell_id }
                                })
                              }
                              oninput={
                                ctx.link().callback(move |ev: InputEvent| {
                                  let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                  let new_value = input.value();

                                  Msg::CellChanged { cell_id, new_value }
                                })
                              }
                              sendinput={
                                ctx.link().callback(move |new_value: String| {
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

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    log_1(&JsValue::from(format!("msg: {msg:?}")));

    match msg {
      Msg::BigInputFocused => {
        match self.input_cell.or(self.prev_focused_cell) {
          Some(cell_id) => {
            self.big_input_text = self.inputs.get(&cell_id).cloned().unwrap_or_default();
            self.focused_cell = Some(cell_id);
          }
          None => (),
        }
        true
      }
      Msg::BigInputChanged { new_value } => match self.input_cell.or(self.focused_cell) {
        Some(cell_id) => {
          self.big_input_text = new_value.clone();
          let expr = parse(&new_value).unwrap_or_else(|_err| Expr::Str(new_value.clone()));
          self.inputs.insert(cell_id, new_value);
          self.exprs.insert(cell_id, expr);

          self.reeval();
          true
        }
        None => true,
      },
      Msg::CellFocused { cell_id, value } => {
        match self.input_cell {
          Some(another_cell_id) if another_cell_id != cell_id => {
            let another_cell_value = self
              .inputs
              .get(&another_cell_id)
              .cloned()
              .unwrap_or_else(|| String::new());

            // formula cell reference insertion
            if another_cell_value.trim_start().starts_with('=') {
              let new_value = format!("{another_cell_value}{}", cell_id.to_string());
              self.big_input_text = new_value.clone();
              ctx.link().send_message(Msg::CellChanged {
                cell_id: another_cell_id,
                new_value,
              });
            } else {
              self.focused_cell = Some(cell_id);
              self.big_input_text = value;
              self.input_cell = None;
            }

            true
          }
          _ => {
            self.focused_cell = Some(cell_id);
            self.big_input_text = value;
            true
          }
        }
      }
      Msg::CellLostFocus { .. } => {
        self.prev_focused_cell = self.focused_cell;
        self.focused_cell = None;
        self.big_input_text = String::from("");
        true
      }
      Msg::CellBecameInput { cell_id } => {
        self.input_cell = Some(cell_id);
        true
      }
      Msg::CellLostInput { .. } => {
        self.input_cell = None;
        true
      }
      Msg::CellChanged { cell_id, new_value } => {
        self.big_input_text = new_value.clone();
        let expr = parse(&new_value).unwrap_or_else(|_err| Expr::Str(new_value.clone()));
        self.inputs.insert(cell_id, new_value);
        self.exprs.insert(cell_id, expr.clone());

        self.reeval();
        true
      }
      Msg::CopyAll => {
        let serialized_cells = self.cells_to_str();

        spawn_local(async move {
          match web_sys::window().unwrap().navigator().clipboard() {
            Some(clipboard) => {
              match JsFuture::from(clipboard.write_text(&serialized_cells)).await {
                Ok(_) => (),
                Err(err) => log_1(&JsValue::from(format!(
                  "couldn't copy cells to clipboard due to {err:?}"
                ))),
              }
            }
            None => log_1(&JsValue::from("cannot access clipboard")),
          }
        });
        true
      }
      Msg::PasteAll => {
        self.paste_modal_visible = true;
        true
      }
      Msg::PasteModalClose => {
        self.paste_modal_visible = false;
        true
      }
      Msg::PasteAllContent { serialized_table } => {
        self.cells_from_str(&serialized_table);
        true
      }
      Msg::Help => {
        // TODO:
        true
      }
    }
  }
}
