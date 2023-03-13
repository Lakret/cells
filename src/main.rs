use std::{collections::HashMap, fmt::Display};

use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone)]
pub struct Cells {
    pub by_id: HashMap<CellId, Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellId {
    col: char,
    row: usize,
}

impl CellId {
    pub fn until<'a>(&'a self, other: &'a Self) -> Box<dyn Iterator<Item = CellId> + 'a> {
        if self.col == other.col {
            // column range
            Box::new((self.row..=other.row).map(|row| CellId { col: self.col, row }))
        } else if self.row == other.row {
            // row range
            Box::new((self.col..=other.col).map(|col| CellId { col, row: self.row }))
        } else {
            // TODO: block range
            todo!()
        }
    }
}

impl Display for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{:02}", self.col, self.row)
    }
}

impl TryFrom<&str> for CellId {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(col) = value.chars().next() {
            if col.is_ascii_uppercase() {
                if let Ok(row) = value.chars().skip(1).collect::<String>().parse() {
                    Ok(CellId { col, row })
                } else {
                    Err("malformed cell id: missing or non-existent row (should be a positive integer)")
                }
            } else {
                Err("malformed cell id: should start with an ASCII uppercase single char column name")
            }
        } else {
            Err("malformed cell id: cannot be empty")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Str(String),
    Num(f64),
    Apply { op: Op, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        if let Ok(num) = value.parse::<f64>() {
            return Expr::Num(num);
        }

        // TODO: Expr, Str
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Expr::*;

    #[test]
    fn cell_id_test() {
        assert_eq!(CellId { col: 'A', row: 18 }.to_string(), "A18");
        assert_eq!(CellId { col: 'Z', row: 1 }.to_string(), "Z01");
        assert_eq!(CellId { col: 'Z', row: 10 }.to_string(), "Z10");
        assert_eq!(CellId { col: 'Z', row: 105 }.to_string(), "Z105");

        assert_eq!(CellId::try_from("A18"), Ok(CellId { col: 'A', row: 18 }));
        assert_eq!(CellId::try_from("Z01"), Ok(CellId { col: 'Z', row: 1 }));
        assert_eq!(
            CellId::try_from(""),
            Err("malformed cell id: cannot be empty")
        );
        assert_eq!(
            CellId::try_from("18"),
            Err("malformed cell id: should start with an ASCII uppercase single char column name")
        );
        assert_eq!(
            CellId::try_from("Z"),
            Err("malformed cell id: missing or non-existent row (should be a positive integer)")
        );
        assert_eq!(
            CellId::try_from("ZZZ"),
            Err("malformed cell id: missing or non-existent row (should be a positive integer)")
        );
    }

    #[test]
    fn expr_parser_test() {
        assert_eq!(Expr::from("12"), Num(12.0));
        assert_eq!(Expr::from("-12"), Num(-12.0));
        assert_eq!(Expr::from("65.98"), Num(65.98));

        // TODO: Apply, strings
    }
}

// TODO: the whole hooks / callbacks thing becomes unwieldy here => use normal TEA
// TODO: selected cell, highlight row and col names on cell selection
#[function_component]
fn App() -> Html {
    let edited = use_state(|| String::new());
    let edited_ref = edited.clone();
    let on_edit_cb = Callback::from(move |val: String| {
        edited.set(val);
    });
    let on_edit_cb_b = &on_edit_cb;

    html! {
        <div class="mx-auto flex flex-col text-white text-xl grow-1">
            <div class="w-screen grow-0 sticky top-0 left-0 z-20 flex gap-4 px-4 py-4 bg-indigo-900">
                <input
                    type="text"
                    value={ (*edited_ref).clone() }
                    class="grow ml-[3rem] px-2 py-0.5 outline-none border-[1px] border-indigo-900 bg-indigo-800"
                />

                <button class="px-4 py-0.5 cursor-pointer rounded-md bg-purple-800 hover:bg-purple-700">
                    { "Clear All" }
                </button>

                <button class="px-4 py-0.5 cursor-pointer rounded-md bg-green-800 hover:bg-green-700">
                    { "Help" }
                </button>
            </div>

            <div class="table table-fixed grow-1 pb-4">
                <thead>
                    <tr>
                        <th class="sticky top-[4.125rem] left-0 pl-6 pr-4 z-10 w-full bg-indigo-900"></th>
                        {
                            ('A'..='Z').map(move |col| {
                                html! {
                                    <th class={
                                        format!(
                                            "{} {}",
                                            "sticky top-[4.125rem] bg-indigo-900",
                                            "text-center text-neutral-400 hover:text-neutral-300"
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
                                            html! {
                                                <th class={
                                                    format!(
                                                        "{} {}",
                                                        "sticky left-0 pl-6 pr-4 bg-indigo-900",
                                                        "text-right text-neutral-400 hover:text-neutral-300"
                                                    )
                                                }>
                                                    { row }
                                                </th>
                                            }
                                        } else {
                                            html! {
                                                <Cell
                                                    cell_id={ CellId { col, row } }
                                                    on_change={ on_edit_cb_b.clone() }
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
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct CellProps {
    cell_id: CellId,
    on_change: Callback<String>,
    #[prop_or_default]
    class: AttrValue,
}

#[function_component]
fn Cell(props: &CellProps) -> Html {
    let cell_val = use_state(|| String::new());

    let on_change = props.on_change.clone();
    let onkeyup = {
        let cell_val = cell_val.clone();
        move |ev: KeyboardEvent| {
            let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
            let val = input.value();
            cell_val.set(val.clone());
            on_change.emit(val);
        }
    };

    let cell_class = "px-2 py-0.5 w-[10rem] outline-none text-right
        border-collapse border-[1px] border-indigo-900 bg-indigo-800";

    html! {
        <td>
            <input
                type="text"
                id={ props.cell_id.to_string() }
                value={ (*cell_val).clone() }
                onkeyup={ onkeyup.clone() }
                class={ format!("{} {}", props.class, cell_class)}
            />
        </td>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
