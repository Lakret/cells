use yew::prelude::*;

use crate::cell_id::CellId;

#[derive(Debug, PartialEq)]
pub enum Msg {
    CellFocused { cell_id: CellId },
}

#[derive(Default)]
pub struct Table {
    big_input_text: String,
    focused_cell: Option<CellId>,
}

static CELL_CLASS: &'static str = "px-2 py-0.5 w-[10rem] outline-none text-right snap-start
border-collapse border-[1px] border-indigo-900 bg-indigo-800";

impl Component for Table {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Table::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mx-auto flex flex-col h-full max-h-full w-full max-w-full text-white text-xl grow-0">
                <div class="w-screen grow-0 sticky top-0 left-0 z-20 flex gap-4 px-4 py-4 bg-indigo-900">
                    // value={ (*top_input_value).clone() }
                    <input
                        type="text"
                        class="grow ml-[3rem] px-2 py-0.5 outline-none border-[1px] border-indigo-900 bg-indigo-800"
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
                                <th class="sticky top-0 left-0 snap-start pl-6 pr-4 z-10 w-full bg-indigo-900">
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
                                                class={
                                                format!(
                                                    "{} {} {}",
                                                    "sticky top-0 snap-start bg-clip-padding bg-indigo-900",
                                                    "text-center",
                                                    header_style
                                                )
                                            }>
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
                                                            format!(
                                                                "{} {} {}",
                                                                "sticky left-0 snap-start pl-6 pr-4 bg-indigo-900",
                                                                "text-right",
                                                                header_style
                                                            )
                                                        }>
                                                            { row }
                                                        </th>
                                                    }
                                                } else {
                                                    let cell_id = { CellId { col, row } };
                                                    // on_change={ on_edit_cb_b.clone() }
                                                    html! {
                                                        <td>
                                                            // id={ props.cell_id.to_string() }
                                                            // value={ (*cell_val).clone() }
                                                            <input
                                                                id={ cell_id.to_string() }
                                                                type="text"
                                                                class={ CELL_CLASS }
                                                                onfocus={
                                                                    ctx.link().callback(move |_ev| Msg::CellFocused {
                                                                        cell_id: cell_id.clone()
                                                                    })
                                                                }
                                                            />
                                                        </td>
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
        // TODO: remove when done
        web_sys::console::log_1(&format!("{msg:?}").into());

        match msg {
            Msg::CellFocused { cell_id } => {
                self.focused_cell = Some(cell_id);
                true
            }
        }
    }
}
