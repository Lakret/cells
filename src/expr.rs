use crate::cell_id::CellId;
use Op::*;

// TODO: (to_evaluate: HashMap<CellId, Expr>) => (deps: HashMap<CellId, Vec<CellId>>)

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Str(String),
  Num(f64),
  CellRef(CellId),
  Apply { op: Op, args: Vec<Expr> },
}

impl Expr {
  pub fn is_value(&self) -> bool {
    match self {
      Expr::Num(_) | Expr::CellRef(_) => true,
      _ => false,
    }
  }
}

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
      Pow => 3,
      Neg => 4,
    }
  }

  pub fn is_left_associative(&self) -> bool {
    if *self != Pow && *self != Neg {
      true
    } else {
      false
    }
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
