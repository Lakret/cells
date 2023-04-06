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

// impl From<&str> for Expr {
//   fn from(value: &str) -> Self {
//     if let Ok(num) = value.parse::<f64>() {
//       return Expr::Num(num);
//     }

//     if let Ok(cell_id) = CellId::try_from(value) {
//       return Expr::CellRef(cell_id);
//     }

//     // TODO: Expr, Str
//     todo!()
//   }
// }

mod parser {
  use nom::branch::alt;
  use nom::bytes::streaming::tag;
  use nom::combinator::map;
  use nom::number::complete::double;
  use nom::IResult;

  use super::*;

  type E<'a> = nom::error::Error<&'a str>;

  fn expr_num(input: &str) -> IResult<&str, Expr> {
    map(double, |num| Expr::Num(num))(input)
  }

  fn expr_op(input: &str) -> IResult<&str, Op> {
    let add = map(tag::<_, _, E>("+"), |_| Op::Add);
    let sub = map(tag::<_, _, E>("-"), |_| Op::Sub);
    let mul = map(tag::<_, _, E>("*"), |_| Op::Mul);
    let div = map(tag::<_, _, E>("/"), |_| Op::Div);

    alt((add, sub, mul, div))(input)
  }

  fn parse_apply(input: &str) -> IResult<&str, Expr> {
    todo!()
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    use crate::expr::Expr::*;

    #[test]
    fn parse_apply_test() {}

    #[test]
    fn expr_parser_test() {
      assert_eq!(expr_num("12"), Ok(("", Num(12.0))));
      assert_eq!(expr_num("-12"), Ok(("", Num(-12.0))));
      assert_eq!(expr_num("65.98"), Ok(("", Num(65.98))));
      assert!(expr_num("sdf").is_err());

      // TODO: Apply, strings
    }
  }
}
