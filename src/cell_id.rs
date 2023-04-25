use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::expr::Expr;

#[derive(Debug, Clone)]
pub struct Cells {
  pub by_id: HashMap<CellId, Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellId {
  pub col: char,
  pub row: usize,
}

impl Display for CellId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}{:02}", self.col, self.row)
  }
}

impl TryFrom<&str> for CellId {
  type Error = &'static str;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    if let Some(col) = value.chars().next() {
      if col.is_ascii_uppercase() {
        if let Ok(row) = value.chars().skip(1).collect::<String>().parse() {
          Ok(CellId { col, row })
        } else {
          Err("malformed cell id: missing or non-existent row (should be a positive integer)")
        }
      } else {
        Err("malformed cell id: should start with an ASCII uppercase single char column name")
      }
    } else {
      Err("malformed cell id: cannot be empty")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cell_id_test() {
    assert_eq!(CellId { col: 'A', row: 18 }.to_string(), "A18");
    assert_eq!(CellId { col: 'Z', row: 1 }.to_string(), "Z01");
    assert_eq!(CellId { col: 'Z', row: 10 }.to_string(), "Z10");
    assert_eq!(CellId { col: 'Z', row: 105 }.to_string(), "Z105");

    assert_eq!(CellId::try_from("A18"), Ok(CellId { col: 'A', row: 18 }));
    assert_eq!(CellId::try_from("Z01"), Ok(CellId { col: 'Z', row: 1 }));
    assert_eq!(
      CellId::try_from(""),
      Err("malformed cell id: cannot be empty")
    );
    assert_eq!(
      CellId::try_from("18"),
      Err("malformed cell id: should start with an ASCII uppercase single char column name")
    );
    assert_eq!(
      CellId::try_from("Z"),
      Err("malformed cell id: missing or non-existent row (should be a positive integer)")
    );
    assert_eq!(
      CellId::try_from("ZZZ"),
      Err("malformed cell id: missing or non-existent row (should be a positive integer)")
    );
  }
}
