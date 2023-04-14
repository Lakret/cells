use std::{collections::HashMap, error::Error};

use crate::cell_id::CellId;
use Op::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Str(String),
  Num(f64),
  CellRef(CellId),
  Apply { op: Op, args: Vec<Expr> },
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
    todo!()
  }
}
