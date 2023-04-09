use std::collections::VecDeque;
use std::error::Error;

use crate::cell_id::CellId;
use crate::expr::{Expr, Op};

// TODO: cell refs and strings
pub fn parse(input: &str) -> Result<Expr, String> {
  todo!()
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Token {
  Op(Op),
  Num(f64),
  LeftParen,
}

fn lex(input: &str) -> impl Iterator<Item = &str> {
  input.split_ascii_whitespace().flat_map(|lexem| {
    let mut lexem = lexem;
    let mut lexems = vec![];

    if lexem.starts_with('(') {
      lexems.push("(");
      lexem = lexem.trim_start_matches('(');
    }

    if lexem.ends_with(')') {
      lexems.push(lexem.trim_end_matches(')'));
      lexems.push(")");
    } else {
      lexems.push(lexem);
    }

    lexems
  })
}

fn shunting_yard(input: &str) -> Result<VecDeque<Token>, Box<dyn Error>> {
  let mut output = VecDeque::new();
  let mut ops = Vec::new();

  for lexem in lex(input) {
    if let Ok(num) = lexem.parse::<f64>() {
      output.push_back(Token::Num(num));
      continue;
    }

    if let Ok(op) = Op::try_from(lexem) {
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
          _ => return Err(format!("impossible token `{top_stack_op:?}` found on the operator stack").into()),
        }
      }

      ops.push(Token::Op(op));
      continue;
    }

    match lexem {
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

#[cfg(test)]
mod tests {
  use super::*;
  // use crate::expr::Expr::*;
  // use crate::expr::Op::*;

  #[test]
  fn shunting_yard_test() {
    use crate::expr::Op::*;
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

  // #[test]
  // fn parse_test() {
  //   assert_eq!(parse("=12"), Ok(Num(12.)));
  //   assert_eq!(parse("=12.2"), Ok(Num(12.2)));
  //   assert_eq!(parse("= -12.2"), Ok(Num(-12.2)));

  //   assert_eq!(
  //     parse("= -12.2 * 4"),
  //     Ok(Apply {
  //       op: Mul,
  //       args: vec![Num(-12.2), Num(4.0)]
  //     })
  //   );

  //   assert_eq!(
  //     parse("= -12.2 - 5"),
  //     Ok(Apply {
  //       op: Sub,
  //       args: vec![Num(-12.2), Num(5.0)]
  //     })
  //   );

  //   assert_eq!(
  //     parse("= 12.2 + 5"),
  //     Ok(Apply {
  //       op: Add,
  //       args: vec![Num(12.2), Num(5.0)]
  //     })
  //   );

  //   assert_eq!(
  //     parse("= 12.2 + 5 / -8.12"),
  //     Ok(Apply {
  //       op: Add,
  //       args: vec![
  //         Num(12.2),
  //         Apply {
  //           op: Div,
  //           args: vec![Num(5.0), Num(-8.12)]
  //         }
  //       ]
  //     })
  //   );

  //   assert_eq!(
  //     parse("=8*12.2*3 + 5 / (-8.12+89.8-8)"),
  //     Ok(Apply {
  //       op: Add,
  //       args: vec![
  //         Apply {
  //           op: Mul,
  //           args: vec![
  //             Apply {
  //               op: Mul,
  //               args: vec![Num(8.0), Num(12.2)]
  //             },
  //             Num(3.0)
  //           ]
  //         },
  //         Apply {
  //           op: Div,
  //           args: vec![
  //             Num(5.0),
  //             Apply {
  //               op: Sub,
  //               args: vec![
  //                 Apply {
  //                   op: Add,
  //                   args: vec![Num(-8.12), Num(89.8)]
  //                 },
  //                 Num(8.0)
  //               ]
  //             }
  //           ]
  //         }
  //       ]
  //     })
  //   );
  // }

  // #[test]
  // fn parse_exr_test() {
  //   assert_eq!(expr("12"), Ok(("", Num(12.0))));
  //   assert_eq!(expr("-12"), Ok(("", Num(-12.0))));
  //   assert_eq!(expr("65.98"), Ok(("", Num(65.98))));

  //   assert_eq!(
  //     expr("6+15"),
  //     Ok((
  //       "",
  //       Apply {
  //         op: Add,
  //         args: vec![Num(6.0), Num(15.0)]
  //       }
  //     ))
  //   );

  //   assert_eq!(
  //     expr("6/ 3.5 + 15"),
  //     Ok((
  //       "",
  //       Apply {
  //         op: Add,
  //         args: vec![
  //           Apply {
  //             op: Div,
  //             args: vec![Num(6.0), Num(3.5)]
  //           },
  //           Num(15.0)
  //         ]
  //       }
  //     ))
  //   );

  //   assert_eq!(
  //     expr("6 / 2 + 15 * 2"),
  //     Ok((
  //       "",
  //       Apply {
  //         op: Add,
  //         args: vec![
  //           Apply {
  //             op: Div,
  //             args: vec![Num(6.0), Num(2.0)]
  //           },
  //           Apply {
  //             op: Mul,
  //             args: vec![Num(15.0), Num(2.0)]
  //           }
  //         ]
  //       }
  //     ))
  //   );

  //   assert_eq!(
  //     expr("6* 3"),
  //     Ok((
  //       "",
  //       Apply {
  //         op: Mul,
  //         args: vec![Num(6.0), Num(3.0)]
  //       }
  //     ))
  //   );

  //   assert_eq!(
  //     expr("6 * 3 / 2"),
  //     Ok((
  //       "",
  //       Apply {
  //         op: Div,
  //         args: vec![
  //           Apply {
  //             op: Mul,
  //             args: vec![Num(6.0), Num(3.0)]
  //           },
  //           Num(2.0)
  //         ]
  //       }
  //     ))
  //   );

  //   assert_eq!(
  //     expr("6 / (2 + 15) * 2"),
  //     Ok((
  //       "",
  //       Apply {
  //         op: Mul,
  //         args: vec![
  //           Apply {
  //             op: Div,
  //             args: vec![
  //               Num(6.0),
  //               Apply {
  //                 op: Add,
  //                 args: vec![Num(2.0), Num(15.0)]
  //               }
  //             ]
  //           },
  //           Num(2.0)
  //         ]
  //       }
  //     ))
  //   );
  // }

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
