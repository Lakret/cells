use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, space0};
use nom::combinator::map;
use nom::multi::fold_many1;
use nom::number::complete::double;
use nom::sequence::{delimited, pair, Tuple};
use nom::IResult;

use crate::expr::{Expr, Op};

pub fn parse(input: &str) -> Result<Expr, String> {
  let (_, (_, _, res)) = (tag("="), space0, expr)
    .parse(input)
    .map_err(|err| err.to_string())?;
  Ok(res)
}

fn op_additive(input: &str) -> IResult<&str, Op> {
  let add = map(char('+'), |_| Op::Add);
  let sub = map(char('-'), |_| Op::Sub);

  alt((add, sub))(input)
}

fn op_multiplicative(input: &str) -> IResult<&str, Op> {
  let mul = map(char('*'), |_| Op::Mul);
  let div = map(char('/'), |_| Op::Div);

  alt((mul, div))(input)
}

fn num(input: &str) -> IResult<&str, Expr> {
  map(delimited(space0, double, space0), |num| Expr::Num(num))(input)
}

fn parens(input: &str) -> IResult<&str, Expr> {
  delimited(space0, delimited(char('('), expr, char(')')), space0)(input)
}

/// Parses expressions like `x*y` and `x/y*z` into an `Expr::Apply`.
fn multiplicative(input: &str) -> IResult<&str, Expr> {
  // takes the first term and keeps the rest in input
  let (input, start) = alt((num, parens))(input)?;

  // for each successive application of `*x` or `/x`,
  // adds the corresponding tuple to the `ops` vector.
  let (rest, ops) = fold_many1(
    pair(op_multiplicative, alt((num, parens))),
    Vec::new,
    |mut acc, (op, val)| {
      acc.push((op, val));
      acc
    },
  )(input)?;

  // At this point, `start` contains the first term
  // and `ops` contains a vector of a form `[(op1, term2), (op2, term3), ...]`.
  // We can now proceed to build a well-formed Apply by starting with transforming the `start` term
  // and the first tuple in the `ops` into a valid Apply (equivalent to `Apply {op: op1, args: [start, term2]}`).
  let mut res = Expr::Apply {
    op: ops[0].0.clone(),
    args: vec![start, ops[0].1.clone()],
  };

  // ... and then we progressively represent each operation in the chain by wrapping it in Apply
  // and using the "result so far" as the first argument of that Apply.
  for (op, arg2) in ops.into_iter().skip(1) {
    res = Expr::Apply {
      op,
      args: vec![res.clone(), arg2],
    }
  }

  Ok((rest, res))
}

/// Parses expressions like `x+y` and `x*y/z-a*b`.
fn additive(input: &str) -> IResult<&str, Expr> {
  let (input, start) = alt((multiplicative, num))(input)?;
  let (rest, ops) = fold_many1(
    pair(op_additive, alt((multiplicative, num))),
    Vec::new,
    |mut acc, (op, val)| {
      acc.push((op, val));
      acc
    },
  )(input)?;

  let mut res = Expr::Apply {
    op: ops[0].0.clone(),
    args: vec![start, ops[0].1.clone()],
  };

  for (op, arg2) in ops.into_iter().skip(1) {
    res = Expr::Apply {
      op,
      args: vec![res.clone(), arg2],
    }
  }

  Ok((rest, res))
}

fn expr(input: &str) -> IResult<&str, Expr> {
  alt((additive, multiplicative, num))(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::Expr::*;
  use crate::expr::Op::*;

  #[test]
  fn parse_test() {
    assert_eq!(parse("=12"), Ok(Num(12.)));
    assert_eq!(parse("=12.2"), Ok(Num(12.2)));
    assert_eq!(parse("= -12.2"), Ok(Num(-12.2)));

    assert_eq!(
      parse("= -12.2 * 4"),
      Ok(Apply {
        op: Mul,
        args: vec![Num(-12.2), Num(4.0)]
      })
    );

    assert_eq!(
      parse("= -12.2 - 5"),
      Ok(Apply {
        op: Sub,
        args: vec![Num(-12.2), Num(5.0)]
      })
    );

    assert_eq!(
      parse("= 12.2 + 5"),
      Ok(Apply {
        op: Add,
        args: vec![Num(12.2), Num(5.0)]
      })
    );

    assert_eq!(
      parse("= 12.2 + 5 / -8.12"),
      Ok(Apply {
        op: Add,
        args: vec![
          Num(12.2),
          Apply {
            op: Div,
            args: vec![Num(5.0), Num(-8.12)]
          }
        ]
      })
    );

    assert_eq!(
      parse("=8*12.2*3 + 5 / (-8.12+89.8-8)"),
      Ok(Apply {
        op: Add,
        args: vec![
          Apply {
            op: Mul,
            args: vec![
              Apply {
                op: Mul,
                args: vec![Num(8.0), Num(12.2)]
              },
              Num(3.0)
            ]
          },
          Apply {
            op: Div,
            args: vec![
              Num(5.0),
              Apply {
                op: Sub,
                args: vec![
                  Apply {
                    op: Add,
                    args: vec![Num(-8.12), Num(89.8)]
                  },
                  Num(8.0)
                ]
              }
            ]
          }
        ]
      })
    );
  }

  #[test]
  fn parse_exr_test() {
    assert_eq!(expr("12"), Ok(("", Num(12.0))));
    assert_eq!(expr("-12"), Ok(("", Num(-12.0))));
    assert_eq!(expr("65.98"), Ok(("", Num(65.98))));

    assert_eq!(
      expr("6+15"),
      Ok((
        "",
        Apply {
          op: Add,
          args: vec![Num(6.0), Num(15.0)]
        }
      ))
    );

    assert_eq!(
      expr("6/ 3.5 + 15"),
      Ok((
        "",
        Apply {
          op: Add,
          args: vec![
            Apply {
              op: Div,
              args: vec![Num(6.0), Num(3.5)]
            },
            Num(15.0)
          ]
        }
      ))
    );

    assert_eq!(
      expr("6 / 2 + 15 * 2"),
      Ok((
        "",
        Apply {
          op: Add,
          args: vec![
            Apply {
              op: Div,
              args: vec![Num(6.0), Num(2.0)]
            },
            Apply {
              op: Mul,
              args: vec![Num(15.0), Num(2.0)]
            }
          ]
        }
      ))
    );

    assert_eq!(
      expr("6* 3"),
      Ok((
        "",
        Apply {
          op: Mul,
          args: vec![Num(6.0), Num(3.0)]
        }
      ))
    );

    assert_eq!(
      expr("6 * 3 / 2"),
      Ok((
        "",
        Apply {
          op: Div,
          args: vec![
            Apply {
              op: Mul,
              args: vec![Num(6.0), Num(3.0)]
            },
            Num(2.0)
          ]
        }
      ))
    );

    assert_eq!(
      expr("6 / (2 + 15) * 2"),
      Ok((
        "",
        Apply {
          op: Mul,
          args: vec![
            Apply {
              op: Div,
              args: vec![
                Num(6.0),
                Apply {
                  op: Add,
                  args: vec![Num(2.0), Num(15.0)]
                }
              ]
            },
            Num(2.0)
          ]
        }
      ))
    );
  }
}
