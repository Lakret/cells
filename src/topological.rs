use crate::{cell_id::CellId, expr::Expr};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Graph<T>(HashMap<T, HashSet<T>>);

impl<T> From<Graph<T>> for HashMap<T, HashSet<T>> {
  fn from(graph: Graph<T>) -> Self {
    graph.0
  }
}

/// Performs topological sorting for a `T` that can be converted to `State<Id>`
/// (`From<T>` is implemented for `State<Id>`).
///
/// ## Implementation Notes
///
/// The following code in this while loop is possible to replace with
/// the following line, but we prefer significantly better readability over
/// slightly better performance (this avoids one clone):
/// `state.resolve_for_dependants_of(&cell_id);`
///
pub fn topological_sort<T, Id>(deps: T) -> Result<Vec<Id>, Box<dyn std::error::Error>>
where
  Id: Eq + std::hash::Hash + Copy + std::fmt::Debug,
  State<Id>: From<T>,
{
  let mut res = vec![];
  let mut state = State::from(deps);

  while let Some(cell_id) = state.no_deps.pop() {
    res.push(cell_id);

    if let Some(dependents) = state.get_dependents(&cell_id) {
      for dependent in dependents.clone() {
        state.resolve(&dependent, &cell_id);
      }
    }
  }

  if !state.is_resolved() {
    return Err(
      format!(
        "cycle or non-computable cell reference detected in cells: {:?}",
        state.unresolved().collect::<Vec<_>>()
      )
      .into(),
    );
  }

  Ok(res)
}

/// Preprocessed state for Kahn's topological sorting algorithm.
///
/// Allows (expected) O(1) dependencies & dependents retrieval for any `node_id: T`
/// and stores `no_deps` vector.
pub struct State<T> {
  // maps a cell_id to a set of cell_ids it depends on
  depends_on: Graph<T>,
  // maps a cell_id to a set of cell_ids depending on it
  dependents: Graph<T>,
  no_deps: Vec<T>,
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

impl<T> State<T>
where
  T: Eq + std::hash::Hash,
{
  pub fn get_dependents(self: &Self, dependency: &T) -> Option<&HashSet<T>> {
    // it's possible to replace the return type with HashSet<T>, but then we'll need to allocate
    self.dependents.0.get(dependency)
  }

  pub fn is_resolved(self: &Self) -> bool {
    self.depends_on.0.is_empty()
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

  pub fn unresolved(self: &Self) -> impl Iterator<Item = &T> {
    self.depends_on.0.keys()
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

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::parse;
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
