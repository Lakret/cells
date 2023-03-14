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
