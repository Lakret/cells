use yew::prelude::*;

use crate::modal::{Modal, ModalProps};

#[function_component]
pub fn HelpModal(props: &ModalProps) -> Html {
  html! {
    <Modal title="Help" is_visible={props.is_visible} onclose={props.onclose.clone()}>
      <div class="flex flex-col gap-2">
        <p>
          {"
            This is a demo of a simple spreadsheet implemented in Rust / WebAssembly with Yew framework.
            It has the following capabilities:
          "}
        </p>
        <ul class="list-inside list-disc">
          <li>{"Select cells with a click."}</li>
          <li>{"Double click, typing with a selected cell, or typing into the big input with a selected cell
          turns a cell into an input."}</li>
          <li>{"Interpret simple formulas starting with = and containing numeric literals, cell references,
          or the following mathematical operations: + - * / ^."}</li>
          <li>{"Dynamically recompute table on cell change."}</li>
          <li>
            {"Copy & paste the content of the table. Here's "}
            <a href="https://github.com/Lakret/cells/blob/main/sample_tables/infrastructure.json"
              class="underline hover:text-sky-300"
              target="_blank">
              {"a table"}
            </a>
            {" you can try."}
          </li>
          <li>{"Enter can be used to confirm cell input and move to the next cell in the same column."}</li>
        </ul>
        <p>
          {"You can see more of my work at "}
          <a href="https://lakret.net" class="underline hover:text-sky-300">
            { "https://lakret.net" }
          </a>
          {". You can see the code for this demo in this repository: "}
          <a href="https://github.com/Lakret/cells" class="underline hover:text-sky-300">
            { "https://github.com/Lakret/cells" }
          </a>
          {"."}
        </p>
      </div>
    </Modal>
  }
}
