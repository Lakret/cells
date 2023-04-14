use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fmt::format,
};

use crate::cell_id::CellId;
use Op::*;

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
      Neg | Pow => 3,
    }
  }

  pub fn is_left_associative(&self) -> bool {
    !(*self == Neg || *self == Pow)
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

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Str(String),
  Num(f64),
  CellRef(CellId),
  Apply { op: Op, args: Vec<Expr> },
}

impl Expr {
  /// Returns a vector of `CellId`s which need to be evaluated before this expression
  /// can be evaluated.
  pub fn get_deps(&self) -> Vec<CellId> {
    let mut deps = vec![];

    let mut stack = vec![self];
    while let Some(expr) = stack.pop() {
      match expr {
        Expr::Str(_) | Expr::Num(_) => (),
        Expr::CellRef(cell_id) => deps.push(cell_id.clone()),
        Expr::Apply { args, .. } => {
          for arg in args {
            stack.push(arg);
          }
        }
      }
    }

    deps
  }

  pub fn eval(&self, ctx: &HashMap<CellId, f64>) -> Result<f64, Box<dyn Error>> {
    match self {
      Expr::Num(num) => Ok(*num),
      Expr::CellRef(cell_id) => ctx.get(cell_id).map(|v| *v).ok_or_else(|| {
        format!("cannot resolve reference to {cell_id:?}")
          .as_str()
          .into()
      }),
      Expr::Apply { op, args } => match op {
        Op::Neg => args[0].eval(ctx).map(|v| -v),
        _ => {
          let args = args
            .iter()
            .map(|arg| arg.eval(ctx))
            .collect::<Result<Vec<_>, _>>()?;

          if args.len() == 2 {
            match op {
              Add => Ok(args[0] + args[1]),
              Sub => Ok(args[0] - args[1]),
              Mul => Ok(args[0] * args[1]),
              Div => Ok(args[0] / args[1]),
              Pow => Ok(args[0].powf(args[1])),
              _ => panic!(
                "programming error: this cannot be reached, since Neg should be handled before"
              ),
            }
          } else {
            Err(
              format!("binary operation {op:?} got incorrect number of arguments: {args:?}")
                .as_str()
                .into(),
            )
          }
        }
      },
      Expr::Str(_) => Err("cannot evaluate strings".into()),
    }
  }
}

pub fn topological_sort(exprs: &HashMap<CellId, Expr>) -> Result<Vec<CellId>, Box<dyn Error>> {
  // maps cell_ids to a vector of cell_ids it depends on
  let mut depends_on: HashMap<_, HashSet<_>> = HashMap::new();
  // maps cell_ids to a vector of cell_ids depending on it
  let mut dependents: HashMap<_, HashSet<_>> = HashMap::new();
  let mut no_deps = vec![];

  for (&cell_id, expr) in exprs.iter() {
    let deps = expr.get_deps();

    if deps.is_empty() {
      no_deps.push(cell_id);
    } else {
      for dep_cell_id in deps {
        depends_on
          .entry(cell_id)
          .and_modify(|dependencies| {
            dependencies.insert(dep_cell_id);
          })
          .or_insert_with(|| {
            let mut s = HashSet::new();
            s.insert(dep_cell_id);
            s
          });

        dependents
          .entry(dep_cell_id)
          .and_modify(|dependents| {
            dependents.insert(cell_id);
          })
          .or_insert_with(|| {
            let mut s = HashSet::new();
            s.insert(cell_id);
            s
          });
      }
    }
  }

  let mut res = vec![];
  while let Some(cell_id) = no_deps.pop() {
    res.push(cell_id);

    if let Some(dependent_cell_ids) = dependents.get(&cell_id) {
      for dependent_cell_id in dependent_cell_ids.iter() {
        if let Some(depends_on_cell_ids) = depends_on.get_mut(dependent_cell_id) {
          depends_on_cell_ids.remove(&cell_id);

          if depends_on_cell_ids.is_empty() {
            no_deps.push(*dependent_cell_id);

            // we are removing resolved cell_ids from depends_on to be able to report cycles
            depends_on.remove(dependent_cell_id);
          }
        }
      }
    }
  }

  if depends_on.is_empty() {
    Ok(res)
  } else {
    Err(format!("cycle detected between cells: {:?}", depends_on.keys()).into())
  }
}

#[cfg(test)]
mod test {
  use crate::parser::parse;

  use super::*;
  use Expr::*;

  #[test]
  fn topolotical_sort_test() {
    let mut exprs = HashMap::new();
    exprs.insert(
      CellId { col: 'A', row: 1 },
      parse("= (B1 / -C1 ^ 2) * 8").unwrap(),
    );
    exprs.insert(CellId { col: 'B', row: 1 }, Num(15.0));
    exprs.insert(CellId { col: 'C', row: 1 }, Num(3.0));

    let ordering = topological_sort(&exprs).unwrap();
    assert_eq!(ordering.len(), 3);
    assert_eq!(*ordering.last().unwrap(), CellId { col: 'A', row: 1 });
  }
}
