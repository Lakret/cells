use crate::cell_id::CellId;

// TODO: (to_evaluate: HashMap<CellId, Expr>) => (deps: HashMap<CellId, Vec<CellId>>)

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Str(String),
  Num(f64),
  CellRef(CellId),
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

    if let Ok(cell_id) = CellId::try_from(value) {
      return Expr::CellRef(cell_id);
    }

    // TODO: Expr, Str
    todo!()
  }
}
