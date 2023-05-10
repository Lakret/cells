use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

use crate::cell_id::CellId;
use crate::topological::topological_sort;
use Op::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
  Neg,
  Add,
  Sub,
  Mul,
  Div,
  Pow,
}

impl Op {
  pub fn precedence(&self) -> u8 {
    match &self {
      Add | Sub => 1,
      Mul | Div => 2,
      Neg | Pow => 3,
    }
  }

  pub fn is_left_associative(&self) -> bool {
    !(*self == Neg || *self == Pow)
  }
}

impl TryFrom<&str> for Op {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "+" => Ok(Add),
      "-" => Ok(Sub),
      "*" => Ok(Mul),
      "/" => Ok(Div),
      "^" => Ok(Pow),
      _ => Err(format!("`{value}` is not a valid operator.")),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
  Str(String),
  Num(f64),
  CellRef(CellId),
  Apply { op: Op, args: Vec<Expr> },
}

impl Default for Expr {
  fn default() -> Self {
    Expr::Str(String::new())
  }
}

impl Expr {
  /// Returns a vector of `CellId`s which need to be evaluated before this expression
  /// can be evaluated.
  pub fn get_deps(&self) -> Vec<CellId> {
    let mut deps = vec![];

    let mut stack = vec![self];
    while let Some(expr) = stack.pop() {
      match expr {
        Expr::Str(_) | Expr::Num(_) => (),
        Expr::CellRef(cell_id) => deps.push(cell_id.clone()),
        Expr::Apply { args, .. } => {
          for arg in args {
            stack.push(arg);
          }
        }
      }
    }

    deps
  }

  pub fn eval(&self, ctx: &HashMap<CellId, f64>) -> Result<f64, Box<dyn Error>> {
    match self {
      Expr::Num(num) => Ok(*num),
      Expr::CellRef(cell_id) => ctx.get(cell_id).map(|v| *v).ok_or_else(|| {
        format!("cannot resolve reference to {cell_id:?}")
          .as_str()
          .into()
      }),
      Expr::Apply { op, args } => match op {
        Op::Neg => args[0].eval(ctx).map(|v| -v),
        _ => {
          let args = args
            .iter()
            .map(|arg| arg.eval(ctx))
            .collect::<Result<Vec<_>, _>>()?;

          if args.len() == 2 {
            match op {
              Add => Ok(args[0] + args[1]),
              Sub => Ok(args[0] - args[1]),
              Mul => Ok(args[0] * args[1]),
              Div => Ok(args[0] / args[1]),
              Pow => Ok(args[0].powf(args[1])),
              _ => panic!(
                "programming error: this cannot be reached, since Neg should be handled before"
              ),
            }
          } else {
            Err(
              format!("binary operation {op:?} got incorrect number of arguments: {args:?}")
                .as_str()
                .into(),
            )
          }
        }
      },
      Expr::Str(_) => Err("cannot evaluate strings".into()),
    }
  }
}

/// Evaluates a parsed cell_id -> expr map, returning a map cell_id -> expr,
/// in which expressions will be replaced by their computed values where possible
pub fn eval(exprs: &HashMap<CellId, Expr>) -> Result<HashMap<CellId, Expr>, Box<dyn Error>> {
  let mut values = HashMap::new();
  let mut computed = HashMap::new();

  for cell_id in topological_sort(exprs)? {
    if let Some(expr) = exprs.get(&cell_id) {
      match expr {
        Expr::Str(_) => {
          computed.insert(cell_id, expr.clone());
        }
        Expr::Num(n) => {
          values.insert(cell_id, *n);
          computed.insert(cell_id, expr.clone());
        }
        Expr::CellRef(another_cell_id) => {
          if let Some(another_value) = values.get(another_cell_id) {
            values.insert(cell_id, *another_value);
          }

          if let Some(another_computed) = computed.get(another_cell_id) {
            computed.insert(cell_id, another_computed.clone());
          } else {
            return Err(
              format!("reference to an empty cell {another_cell_id} in cell {cell_id}").into(),
            );
          }
        }
        Expr::Apply { .. } => {
          let value = expr.eval(&values)?;
          values.insert(cell_id, value);
          computed.insert(cell_id, Expr::Num(value));
        }
      }
    }
  }

  Ok(computed)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::parse;

  #[test]
  fn expr_eval_test() {
    let expr = parse("= A1 - (A2 - A3 ^ B1 / 2.5) + C1").unwrap();
    let ctx = HashMap::from_iter(vec![
      (CellId { col: 'A', row: 1 }, 12.0),
      (CellId { col: 'A', row: 2 }, 500.5),
      (CellId { col: 'A', row: 3 }, -3.1415),
      (CellId { col: 'B', row: 1 }, 2.0),
      (CellId { col: 'C', row: 1 }, 0.2187456),
    ]);
    assert_eq!(expr.eval(&ctx).unwrap(), -484.33364550000005);
  }
}
