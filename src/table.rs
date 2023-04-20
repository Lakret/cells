use regex::internal::Input;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::*;
use web_sys::console::log_1;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::cell_id::CellId;
use crate::expr::{eval, Expr};
use crate::parser::parse;

#[derive(Debug, PartialEq)]
pub enum Msg {
  CopyAll,
  PasteAll,
  PasteAllContent { serialized_table: String },
  PasteModalClose,
  Help,
  CellFocused { cell_id: CellId, value: String },
  CellLostFocus { cell_id: CellId },
  CellChanged { cell_id: CellId, new_value: String },
}

#[derive(Default, Debug)]
pub struct Table {
  big_input_text: String,
  focused_cell: Option<CellId>,
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
            value={ self.big_input_text.clone() }
            class={classes!(vec![
              "grow ml-[3rem] px-2 py-0.5 outline-none font-mono",
              "border-[1px] border-indigo-900 bg-indigo-800"
            ])}
          />

          <Btn
            title="Copy All"
            color={ BtnColors::Purple }
            onclick={ ctx.link().callback(move |_ev:MouseEvent| { Msg::CopyAll }) }
          />
          <Btn
            title="Paste All"
            color={ BtnColors::Violet }
            onclick={ ctx.link().callback(move |_ev:MouseEvent| { Msg::PasteAll }) }
          />
          <Btn
            title="Help"
            color={ BtnColors::Green }
            onclick={ ctx.link().callback(move |_ev:MouseEvent| { Msg::Help }) }
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
                              input={self.inputs.get(&cell_id).map(|x| x.clone())}
                              expr={self.exprs.get(&cell_id).map(|x| x.clone())}
                              computed={self.computed.get(&cell_id).map(|x| x.clone())}
                              onfocus={
                                ctx.link().callback(move |ev: FocusEvent| {
                                  let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                  let value = input.value();

                                  Msg::CellFocused { cell_id, value }
                                })
                              }
                              onfocusout={
                                ctx.link().callback(move |_ev: FocusEvent| {
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
fn Cell(props: &CellProps) -> Html {
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

#[derive(PartialEq, Properties)]
struct BtnProps {
  pub title: String,
  pub color: BtnColors,
  pub onclick: Callback<MouseEvent>,
}

#[derive(PartialEq)]
enum BtnColors {
  Purple,
  Green,
  Violet,
}

impl BtnColors {
  pub fn to_classes(&self) -> &'static str {
    match self {
      BtnColors::Purple => "bg-purple-800 hover:bg-purple-700",
      BtnColors::Green => "bg-emerald-800 hover:bg-emerald-700",
      BtnColors::Violet => "bg-violet-800 hover:bg-violet-700",
    }
  }
}

#[function_component]
fn Btn(props: &BtnProps) -> Html {
  html! {
    <button
      onclick={props.onclick.clone()}
      class={classes!(vec![
        "flex items-center justify-center leading-none px-4 py-2 cursor-pointer rounded-md",
        props.color.to_classes()
      ])}
    >
      { props.title.clone() }
    </button>
  }
}

#[derive(PartialEq, Properties)]
struct PasteModalProps {
  is_visible: bool,
  onpaste: Callback<String>,
  onclose: Callback<()>,
}

#[function_component]
fn PasteModal(props: &PasteModalProps) -> Html {
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
