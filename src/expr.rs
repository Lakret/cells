use serde::{Deserialize, Serialize};
use std::{
  collections::{HashMap, HashSet},
  error::Error,
};

use crate::cell_id::CellId;
use Op::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
  Str(String),
  Num(f64),
  CellRef(CellId),
  Apply { op: Op, args: Vec<Expr> },
}

impl Default for Expr {
  fn default() -> Self {
    Expr::Str(String::new())
  }
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

/// Evaluates a parsed cell_id -> expr map, returning a map cell_id -> expr,
/// in which expressions will be replaced by their computed values where possible
pub fn eval(exprs: &HashMap<CellId, Expr>) -> Result<HashMap<CellId, Expr>, Box<dyn Error>> {
  let mut values = HashMap::new();
  let mut computed = HashMap::new();

  for cell_id in topological_sort(exprs)? {
    if let Some(expr) = exprs.get(&cell_id) {
      match expr {
        Expr::Str(_) => {
          computed.insert(cell_id, expr.clone());
        }
        Expr::Num(n) => {
          values.insert(cell_id, *n);
          computed.insert(cell_id, expr.clone());
        }
        Expr::CellRef(another_cell_id) => {
          if let Some(another_value) = values.get(another_cell_id) {
            values.insert(cell_id, *another_value);
          }

          if let Some(another_computed) = computed.get(another_cell_id) {
            computed.insert(cell_id, another_computed.clone());
          } else {
            return Err(
              format!("reference to an empty cell {another_cell_id} in cell {cell_id}").into(),
            );
          }
        }
        Expr::Apply { .. } => {
          let value = expr.eval(&values)?;
          values.insert(cell_id, value);
          computed.insert(cell_id, Expr::Num(value));
        }
      }
    }
  }

  Ok(computed)
}

#[derive(Default)]
struct Graph<T>(HashMap<T, HashSet<T>>);

impl<T> From<Graph<T>> for HashMap<T, HashSet<T>> {
  fn from(graph: Graph<T>) -> Self {
    graph.0
  }
}

/// Preprocessed state for Kahn's topological sorting algorithm.
///
/// Allows (expected) O(1) dependencies & dependents retrieval for any `node_id: T`
/// and stores `no_deps` vector.
struct State<T> {
  // maps a cell_id to a set of cell_ids it depends on
  pub depends_on: Graph<T>,
  // maps a cell_id to a set of cell_ids depending on it
  pub dependents: Graph<T>,
  pub no_deps: Vec<T>,
}

impl<T> Default for State<T> {
  fn default() -> Self {
    Self {
      depends_on: Graph(HashMap::new()),
      dependents: Graph(HashMap::new()),
      no_deps: vec![],
    }
  }
}

impl From<&HashMap<CellId, Expr>> for State<CellId> {
  fn from(exprs: &HashMap<CellId, Expr>) -> State<CellId> {
    let mut graphs = State::default();

    for (&cell_id, expr) in exprs.iter() {
      let deps = expr.get_deps();

      if deps.is_empty() {
        graphs.no_deps.push(cell_id);
      } else {
        for dep_cell_id in deps {
          graphs
            .depends_on
            .0
            .entry(cell_id)
            .and_modify(|dependencies| {
              dependencies.insert(dep_cell_id);
            })
            .or_insert_with(|| {
              let mut s = HashSet::new();
              s.insert(dep_cell_id);
              s
            });

          graphs
            .dependents
            .0
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

    graphs
  }
}

impl<T> State<T>
where
  T: Eq + std::hash::Hash,
{
  pub fn get_dependents(self: &Self, dependency: &T) -> Option<&HashSet<T>> {
    // it's possible to replace the return type with HashSet<T>, but then we'll need to allocate
    self.dependents.0.get(dependency)
  }
}

impl<T> State<T>
where
  T: Copy + Eq + std::hash::Hash,
{
  pub fn resolve(self: &mut Self, dependent: &T, dependency: &T) {
    if let Some(dependencies) = self.depends_on.0.get_mut(dependent) {
      dependencies.remove(&dependency);

      if dependencies.is_empty() {
        self.no_deps.push(*dependent);

        // we are removing resolved cell_ids from depends_on to be able to report cycles
        self.depends_on.0.remove(dependent);
      }
    }
  }

  #[allow(dead_code)]
  pub fn resolve_for_dependants_of(self: &mut Self, dependency: &T) {
    if let Some(dependents) = self.dependents.0.get(dependency) {
      for dependent in dependents.iter() {
        // self.resolve(dependent, dependency);
        if let Some(dependencies) = self.depends_on.0.get_mut(dependent) {
          dependencies.remove(&dependency);

          if dependencies.is_empty() {
            self.no_deps.push(*dependent);
            // we are removing resolved cell_ids from depends_on to be able to report cycles
            self.depends_on.0.remove(dependent);
          }
        }
      }
    }
  }
}

fn topological_sort(exprs: &HashMap<CellId, Expr>) -> Result<Vec<CellId>, Box<dyn Error>> {
  let mut state = State::from(exprs);

  let mut res = vec![];
  while let Some(cell_id) = state.no_deps.pop() {
    res.push(cell_id);

    // the following code in this while loop is possible to replace with
    // the following line, but we prefer significantly better readability over
    // slightly better performance (this avoids one clone)
    //
    // state.resolve_for_dependants_of(&cell_id);
    //
    if let Some(dependents) = state.get_dependents(&cell_id) {
      for dependent in dependents.clone() {
        state.resolve(&dependent, &cell_id);
      }
    }
  }

  if state.depends_on.0.is_empty() {
    Ok(res)
  } else {
    Err(
      format!(
        "cycle or non-computable cell reference detected in cells: {:?}",
        state.depends_on.0.keys()
      )
      .into(),
    )
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

  #[test]
  fn expr_eval_test() {
    let expr = parse("= A1 - (A2 - A3 ^ B1 / 2.5) + C1").unwrap();
    let ctx = HashMap::from_iter(vec![
      (CellId { col: 'A', row: 1 }, 12.0),
      (CellId { col: 'A', row: 2 }, 500.5),
      (CellId { col: 'A', row: 3 }, -3.1415),
      (CellId { col: 'B', row: 1 }, 2.0),
      (CellId { col: 'C', row: 1 }, 0.2187456),
    ]);
    assert_eq!(expr.eval(&ctx).unwrap(), -484.33364550000005);
  }
}
