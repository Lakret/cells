use std::{collections::HashMap, fmt::Display, str::FromStr};

use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone)]
struct Cells {
    by_id: HashMap<CellId, Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CellId {
    col: char,
    row: usize,
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
enum Expr {
    Str(String),
    Num(f64),
    Apply { op: Op, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op {
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

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let cell_val = use_state(|| String::new());
    let onkeyup = {
        let cell_val = cell_val.clone();
        move |ev: KeyboardEvent| {
            let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
            cell_val.set(input.value());
        }
    };

    html! {
        <div class="mx-auto container py-10 text-white text-xl grow-1">
            <div class="flex flex-row flex-wrap">
            {
                (1..=10).map(|_input_idx| html! {
                    <input type="text"
                        value={ (*cell_val).clone() }
                        onkeyup={onkeyup.clone()}
                        class="bg-indigo-800 outline-none px-2 py-0.5" />
                }).collect::<Html>()
            }
            </div>

            <p>{ (*cell_val).clone() }</p>

            <br/>
            <button class="rounded-md cursor-pointer" {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
