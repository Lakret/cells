use regex::Regex;
use std::collections::VecDeque;

use crate::cell_id::CellId;
use crate::expr::{Expr, Op};

pub fn parse(input: &str) -> Result<Expr, String> {
  if input.trim().starts_with('=') {
    let tokens = shunting_yard(input.trim().trim_start_matches('='))?;
    to_ast(&tokens)
  } else {
    match input.trim().parse::<f64>() {
      Ok(n) => Ok(Expr::Num(n)),
      Err(_) => Ok(Expr::Str(input.into())),
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Token {
  Op(Op),
  Num(f64),
  CellRef(CellId),
  LeftParen,
}

fn shunting_yard(input: &str) -> Result<VecDeque<Token>, String> {
  let mut output = VecDeque::new();
  let mut ops = Vec::new();

  // used to differentiate negation & subtraction
  let mut prev_token = None;
  for lexem in lex(input) {
    if let Ok(num) = lexem.parse::<f64>() {
      let token = Token::Num(num);
      prev_token = Some(token);
      output.push_back(Token::Num(num));
      continue;
    }

    if let Ok(op) = Op::try_from(lexem) {
      // convert Sub to Neg if it's:
      // - the very start of the input (such as `-15` or `-B5`)
      // - right after the left parenthesis or binary op token (such as `14 - (- 8)` - the 1st is Sub, the 2nd is Neg)
      let is_negation = op == Op::Sub
        && match prev_token {
          None => true,
          Some(Token::Op(op)) if op != Op::Neg => true,
          Some(Token::LeftParen) => true,
          Some(_) => false,
        };
      let op = if is_negation { Op::Neg } else { op };

      while let Some(top_stack_op) = ops.pop() {
        match top_stack_op {
          // stop popping once a left parenthesis is encountered
          Token::LeftParen => {
            ops.push(top_stack_op);
            break;
          }
          Token::Op(top_stack_op_inner) => {
            // push operators with greater precedence
            // or same precedence, but when the current operator is left-associative, to the output
            if top_stack_op_inner.precedence() > op.precedence()
              || (op.is_left_associative() && top_stack_op_inner.precedence() == op.precedence())
            {
              output.push_back(top_stack_op);
            } else {
              ops.push(top_stack_op);
              break;
            }
          }
          _ => {
            return Err(
              format!("impossible token `{top_stack_op:?}` found on the operator stack").into(),
            )
          }
        }
      }

      let token = Token::Op(op);
      prev_token = Some(token);
      ops.push(token);
      continue;
    }

    match lexem {
      "(" => {
        let token = Token::LeftParen;
        prev_token = Some(token);
        ops.push(token);
      }
      ")" => loop {
        match ops.pop() {
          Some(top_stack_op) => match top_stack_op {
            Token::LeftParen => break,
            token => output.push_back(token),
          },
          None => return Err("mismatched parenthesis".into()),
        }
      },
      other => match CellId::try_from(other) {
        Ok(cell_id) => {
          let token = Token::CellRef(cell_id);
          prev_token = Some(token);
          output.push_back(token);
        }
        Err(_) => return Err(format!("unknown lexem `{other}` in `{input}`").into()),
      },
    }
  }

  while let Some(op) = ops.pop() {
    if op == Token::LeftParen {
      return Err("mismatched parenthesis".into());
    }

    output.push_back(op);
  }

  Ok(output)
}

lazy_static! {
  static ref SEP_RE: Regex = Regex::new(r"\s*(?P<op>[*+/()^-])\s*").unwrap();
}

fn lex(input: &str) -> Vec<&str> {
  let mut loc = 0;
  let mut res = vec![];

  for sep in SEP_RE.find_iter(input) {
    if sep.start() > loc {
      res.push(input[loc..sep.start()].trim());
    }
    loc = sep.end();

    res.push(sep.as_str().trim());
  }

  if loc < input.len() {
    res.push(&input[loc..].trim())
  }

  res
}

fn to_ast(tokens: &VecDeque<Token>) -> Result<Expr, String> {
  let empty_stack_op_msg = "empty stack when trying to build operator's AST";
  let mut stack = vec![];

  for token in tokens {
    match token {
      Token::Num(num) => stack.push(Expr::Num(*num)),
      Token::CellRef(cell_id) => stack.push(Expr::CellRef(*cell_id)),
      Token::Op(Op::Neg) => {
        let arg = stack.pop().ok_or(empty_stack_op_msg)?;
        let op = Expr::Apply {
          op: Op::Neg,
          args: vec![arg],
        };
        stack.push(op);
      }
      Token::Op(op) => {
        let right = stack.pop().ok_or(empty_stack_op_msg)?;
        let left = stack.pop().ok_or(empty_stack_op_msg)?;
        let op = Expr::Apply {
          op: *op,
          args: vec![left, right],
        };
        stack.push(op);
      }
      Token::LeftParen => {
        return Err("encountered left parenthesis in the shunting yard output".into())
      }
    }
  }

  match stack.pop() {
    Some(expr) => Ok(expr),
    None => Err(format!("empty stack encountered when building AST for tokens {tokens:?}").into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::Expr;
  use crate::expr::Op::*;

  #[test]
  fn shunting_yard_test() {
    use Token::*;

    assert_eq!(
      shunting_yard("12 + 5 ^ 3").unwrap(),
      VecDeque::from(vec![Num(12.0), Num(5.0), Num(3.0), Op(Pow), Op(Add)])
    );

    assert_eq!(
      shunting_yard("12 + 5 ^ 3 - 8 / 2 * 3.5 + 6.5").unwrap(),
      VecDeque::from(vec![
        Num(12.0),
        Num(5.0),
        Num(3.0),
        Op(Pow),
        Op(Add),
        Num(8.0),
        Num(2.0),
        Op(Div),
        Num(3.5),
        Op(Mul),
        Op(Sub),
        Num(6.5),
        Op(Add)
      ])
    );

    assert_eq!(
      shunting_yard("(12 + 5) ^ 3").unwrap(),
      VecDeque::from(vec![Num(12.0), Num(5.0), Op(Add), Num(3.0), Op(Pow)])
    );

    assert_eq!(
      shunting_yard("12 + 5 ^ (3 - 8 / 2 * 3.5) + 6.5").unwrap(),
      VecDeque::from(vec![
        Num(12.0),
        Num(5.0),
        Num(3.0),
        Num(8.0),
        Num(2.0),
        Op(Div),
        Num(3.5),
        Op(Mul),
        Op(Sub),
        Op(Pow),
        Op(Add),
        Num(6.5),
        Op(Add)
      ])
    );

    assert_eq!(
      shunting_yard("12 + 5 ^ (3 - 8 / 2 * 3.5 + 6.5")
        .unwrap_err()
        .to_string(),
      "mismatched parenthesis"
    );

    assert_eq!(
      shunting_yard("12 + 5 ^ (3 - 8 / 2 * (3.5) + 6.5")
        .unwrap_err()
        .to_string(),
      "mismatched parenthesis"
    );

    assert_eq!(
      shunting_yard("12 + 5 ^ 3 - 8 / 2 * 3.5) + 6.5")
        .unwrap_err()
        .to_string(),
      "mismatched parenthesis"
    );
  }

  #[test]
  fn to_ast_test() {
    assert_eq!(
      to_ast(&VecDeque::from(vec![
        Token::Num(12.0),
        Token::Num(5.0),
        Token::Num(3.0),
        Token::Op(Pow),
        Token::Op(Add)
      ]))
      .unwrap(),
      Expr::Apply {
        op: Add,
        args: vec![
          Expr::Num(12.0),
          Expr::Apply {
            op: Pow,
            args: vec![Expr::Num(5.0), Expr::Num(3.0)]
          }
        ]
      }
    );
  }

  #[test]
  fn parse_test() {
    use Expr::*;

    assert_eq!(parse("12"), Ok(Num(12.0)));
    assert_eq!(parse("yo"), Ok(Str("yo".to_string())));

    assert_eq!(parse("A12"), Ok(Str("A12".to_string())));
    assert_eq!(parse("= A12"), Ok(CellRef(CellId { col: 'A', row: 12 })));

    assert_eq!(parse("=12"), Ok(Num(12.)));
    assert_eq!(parse("=12.2"), Ok(Num(12.2)));
    assert_eq!(
      parse("= -12.2"),
      Ok(Apply {
        op: Neg,
        args: vec![Num(12.2)]
      })
    );

    assert_eq!(
      parse("= -12.2 * 4"),
      Ok(Apply {
        op: Mul,
        args: vec![
          Apply {
            op: Neg,
            args: vec![Num(12.2)]
          },
          Num(4.0)
        ]
      })
    );

    assert_eq!(
      parse("= -12.2 - 5"),
      Ok(Apply {
        op: Sub,
        args: vec![
          Apply {
            op: Neg,
            args: vec![Num(12.2)]
          },
          Num(5.0)
        ]
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
            args: vec![
              Num(5.0),
              Apply {
                op: Neg,
                args: vec![Num(8.12)]
              }
            ]
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
                    args: vec![
                      Apply {
                        op: Neg,
                        args: vec![Num(8.12)]
                      },
                      Num(89.8)
                    ]
                  },
                  Num(8.0)
                ]
              }
            ]
          }
        ]
      })
    );

    assert_eq!(
      parse("= 12.2 + A5"),
      Ok(Apply {
        op: Add,
        args: vec![Num(12.2), CellRef(CellId { col: 'A', row: 5 })]
      })
    );

    assert_eq!(
      parse("=K12*12.2*3 + 5 / (-8.12+B5-8)"),
      Ok(Apply {
        op: Add,
        args: vec![
          Apply {
            op: Mul,
            args: vec![
              Apply {
                op: Mul,
                args: vec![CellRef(CellId { col: 'K', row: 12 }), Num(12.2)]
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
                    args: vec![
                      Apply {
                        op: Neg,
                        args: vec![Num(8.12)]
                      },
                      CellRef(CellId { col: 'B', row: 5 })
                    ]
                  },
                  Num(8.0)
                ]
              }
            ]
          }
        ]
      })
    );

    assert_eq!(
      parse("= -C15 - -A5 - (-B5 - (-3.1415 + -C1))"),
      Ok(Apply {
        op: Sub,
        args: vec![
          Apply {
            op: Sub,
            args: vec![
              Apply {
                op: Neg,
                args: vec![CellRef(CellId { col: 'C', row: 15 })]
              },
              Apply {
                op: Neg,
                args: vec![CellRef(CellId { col: 'A', row: 5 })]
              }
            ]
          },
          Apply {
            op: Sub,
            args: vec![
              Apply {
                op: Neg,
                args: vec![CellRef(CellId { col: 'B', row: 5 })]
              },
              Apply {
                op: Add,
                args: vec![
                  Apply {
                    op: Neg,
                    args: vec![Num(3.1415)]
                  },
                  Apply {
                    op: Neg,
                    args: vec![CellRef(CellId { col: 'C', row: 1 })]
                  }
                ]
              }
            ]
          }
        ]
      })
    );
  }
}
