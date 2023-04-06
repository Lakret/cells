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
  use nom::sequence::Tuple;
  use nom::IResult;

  use super::*;

  type E<'a> = nom::error::Error<&'a str>;

  fn num(input: &str) -> IResult<&str, Expr> {
    map(double, |num| Expr::Num(num))(input)
  }

  fn op_additive(input: &str) -> IResult<&str, Op> {
    let add = map(tag("+"), |_| Op::Add);
    let sub = map(tag("-"), |_| Op::Sub);

    alt((add, sub))(input)
  }

  fn op_multiplicative(input: &str) -> IResult<&str, Op> {
    let mul = map(tag("*"), |_| Op::Mul);
    let div = map(tag("/"), |_| Op::Div);

    alt((mul, div))(input)
  }

  fn op(input: &str) -> IResult<&str, Op> {
    alt((op_additive, op_multiplicative))(input)
  }

  // expressions like `x + y` and `x - y`
  fn additive(input: &str) -> IResult<&str, Expr> {
    let (rest, (left, op, right)) = (num, op_additive, num).parse(input)?;
    Ok((
      rest,
      Expr::Apply {
        op,
        args: vec![left, right],
      },
    ))
  }

  fn parse_apply(input: &str) -> IResult<&str, Expr> {
    todo!()
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    use crate::expr::Expr::*;

    #[test]
    fn parse_op_test() {
      assert_eq!(op("+"), Ok(("", Op::Add)));
      assert_eq!(op("-"), Ok(("", Op::Sub)));
      assert_eq!(op("*"), Ok(("", Op::Mul)));
      assert_eq!(op("/"), Ok(("", Op::Div)));

      assert_eq!(op("**"), Ok(("*", Op::Mul)));
      assert!(op("_").is_err());
    }

    #[test]
    fn parse_additive_test() {
      assert_eq!(
        additive("12+85"),
        Ok((
          "",
          Expr::Apply {
            op: Op::Add,
            args: vec![Num(12.0), Num(85.0)]
          }
        ))
      );

      assert_eq!(
        additive("58.28-85.123"),
        Ok((
          "",
          Expr::Apply {
            op: Op::Sub,
            args: vec![Num(58.28), Num(85.123)]
          }
        ))
      );
    }

    #[test]
    fn expr_parser_test() {
      assert_eq!(num("12"), Ok(("", Num(12.0))));
      assert_eq!(num("-12"), Ok(("", Num(-12.0))));
      assert_eq!(num("65.98"), Ok(("", Num(65.98))));
      assert!(num("sdf").is_err());

      // TODO: Apply, strings
    }
  }
}
