use regex::Regex;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::cell_id::CellId;
use crate::expr::{Expr, Op};

// TODO: cell refs
pub fn parse(input: &str) -> Result<Expr, String> {
  if input.trim().starts_with('=') {
    let tokens = shunting_yard(input.trim().trim_start_matches('='))?;
    to_ast(&tokens)
  } else {
    Ok(Expr::Str(input.into()))
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Token {
  Op(Op),
  Num(f64),
  LeftParen,
}

fn shunting_yard(input: &str) -> Result<VecDeque<Token>, String> {
  let mut output = VecDeque::new();
  let mut ops = Vec::new();

  for lexem in unary_minus_to_negative_numbers(lex(input)) {
    if let Ok(num) = lexem.parse::<f64>() {
      output.push_back(Token::Num(num));
      continue;
    }

    if let Ok(op) = Op::try_from(lexem.as_str()) {
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

      ops.push(Token::Op(op));
      continue;
    }

    match lexem.as_str() {
      "(" => ops.push(Token::LeftParen),
      ")" => loop {
        match ops.pop() {
          Some(top_stack_op) => match top_stack_op {
            Token::LeftParen => break,
            token => output.push_back(token),
          },
          None => return Err("mismatched parenthesis".into()),
        }
      },
      unknown_lexem => return Err(format!("unknown lexem `{unknown_lexem}` in `{input}`").into()),
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
  static ref SEP_RE: Regex = Regex::new(r"\s*(?P<op>[*+/()-])\s*").unwrap();
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

  if loc < input.len() - 1 {
    res.push(&input[loc..].trim())
  }

  dbg!(res)
}

// process negative numbers by combining them with the preceding minus sign
// in case this minus cannot be a binary operation
fn unary_minus_to_negative_numbers(lexems: Vec<&str>) -> Vec<String> {
  let mut res = vec![];
  let mut preceded_by_minus = false;

  // TODO: doesn't work as expected, because windows doesn't return shorter windows
  // at all :(
  for (idx, window) in lexems.windows(3).enumerate() {
    match window {
      &[] => (),
      &[lexem] => res.push(lexem.to_string()),
      &["-", lexem] if lexem.parse::<f64>().is_ok() => {
        res.push(format!("-{lexem}"));
      }
      &[lexem1, lexem2] => {
        res.push(lexem1.to_string());
        res.push(lexem2.to_string());
      }
      &[grandparent, parent, lexem] => {
        if idx == 0 {
          // process the first lexem
          preceded_by_minus = grandparent == "-";
          if !preceded_by_minus {
            res.push(grandparent.to_string());
          }

          // process the second lexem
          if preceded_by_minus && parent.parse::<f64>().is_ok() {
            res.push(format!("-{parent}"));
          }
          preceded_by_minus = parent == "-";
        }

        if preceded_by_minus {
          // if preceded by minus, can be parsed as a number, and the grandparent is a separator,
          // recognize as a negative number;
          // otherwise, push both the minus and the lexem into the output
          if lexem.parse::<f64>().is_ok() && SEP_RE.is_match(grandparent) {
            res.push(format!("-{lexem}"))
          } else {
            res.push("-".to_string());
            res.push(lexem.to_string());
          }
        } else {
          if lexem != "-" {
            res.push(lexem.to_string())
          }
        }

        preceded_by_minus = lexem == "-";
      }
      _ => (),
    }
  }

  dbg!(res)
}

fn to_ast(tokens: &VecDeque<Token>) -> Result<Expr, String> {
  let empty_stack_op_msg = "empty stack when trying to build operator's AST";
  let mut stack = vec![];

  for token in tokens {
    match token {
      Token::Num(num) => stack.push(Expr::Num(*num)),
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
    None => Err("empty stack encountered when building AST".into()),
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

    dbg!(lex("=8*12.2*3 + 5 / (-8.12+89.8-8)"));

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

  // #[test]
  // fn parse_cell_ref_test() {
  //   assert_eq!(
  //     cell_ref("A15"),
  //     Ok(("", CellRef(CellId { col: 'A', row: 15 })))
  //   );
  //   assert_eq!(
  //     cell_ref("Z20"),
  //     Ok(("", CellRef(CellId { col: 'Z', row: 20 })))
  //   );

  //   assert!(cell_ref("AF20").is_err());
  //   assert_eq!(
  //     cell_ref("20"),
  //     Ok(("", CellRef(CellId { col: 'Z', row: 20 })))
  //   );
  // }
}
