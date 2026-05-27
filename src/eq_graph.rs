//! Equivalence graph and expression Ids.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Index;
use std::{fmt, mem};

use rustc_hash::FxBuildHasher as BuildHasher;

use crate::ty::Type;

/// A unique high-level-representation Id to an expression [`Type`].
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct HirId(usize);

/// A class of equivalent [`Type`] expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HirClass {
  /// Equivalent nodes Ids.
  nodes: Vec<HirId>,
  /// Dependent parents Ids.
  pdeps: Vec<HirId>,
}

/// A graph maintaining equivalence between [`Type`] expressions.
#[derive(Default, Debug, Clone)]
pub struct Graph<T, K> {
  /// A union-find equivalence structure.
  parent: Vec<HirId>,
  /// A map indexed on an IR arena.
  ty_map: Vec<K>,
  /// A map from IR to unique Id.
  id_map: HashMap<K, HirId, BuildHasher>,
  /// A quotient space of equivalence classes.
  qspace: HashMap<HirId, HirClass, BuildHasher>,
  /// A congruence tracker.
  cong: HashSet<HirId, BuildHasher>,
  /// A worklist for rebuilds.
  work: Vec<HirId>,
  /// A type marker.
  _ty: PhantomData<T>,
}

impl<T, K> Graph<T, K>
where
  T: Type<Term<HirId> = K>,
  K: Clone + Eq + Hash,
{
  /// Constructs a new, empty [`Graph`].
  pub fn new() -> Self {
    Graph {
      parent: Vec::new(),
      ty_map: Vec::new(),
      id_map: HashMap::default(),
      qspace: HashMap::default(),
      cong: HashSet::default(),
      work: Vec::new(),
      _ty: PhantomData,
    }
  }

  /// Find the root (minimal) equivalent expression.
  pub fn find(
    // cur_id < n
    &mut self,
    cur_id: HirId,
  ) -> HirId {
    assert!(cur_id.0 < self.parent.len());
    if cur_id != self.parent[cur_id.0] {
      let path_id = self.parent[cur_id.0];
      self.parent[cur_id.0] = self.parent[path_id.0];
      self.find(path_id)
    } else {
      cur_id
    }
  }

  pub(crate) fn search(
    // cur_id < n
    &self,
    mut cur_id: HirId,
  ) -> HirId {
    assert!(cur_id.0 < self.parent.len());
    while cur_id != self.parent[cur_id.0] {
      // cur_id = parent
      // min_{cur_id}
      cur_id = self.parent[cur_id.0];
    }

    cur_id
  }

  /// Merge two expressions by their union.
  pub fn union(
    // lhs_id < n
    // rhs_id < n
    &mut self,
    lhs_id: HirId,
    rhs_id: HirId,
  ) -> Option<HirId> {
    assert!(lhs_id.0 < self.parent.len());
    assert!(rhs_id.0 < self.parent.len());

    let mut lhs_id = self.find(lhs_id);
    let mut rhs_id = self.find(rhs_id);

    if lhs_id != rhs_id {
      // min_{lhs, rhs}
      if lhs_id.0 > rhs_id.0 {
        mem::swap(&mut lhs_id, &mut rhs_id);
      }

      // lhs (oldest)
      // = parent
      // = rhs
      self.parent[rhs_id.0] = lhs_id;
      if let Some((mut rhs_class, lhs_class)) = self.qspace.remove(&rhs_id).zip(self.qspace.get_mut(&lhs_id)) {
        self.work.extend(rhs_class.pdeps.iter());

        lhs_class.nodes.append(
          // rhs ⊆ lhs
          &mut rhs_class.nodes,
        );
      }
      Some(lhs_id)
    } else {
      None
    }
  }

  /// Restore invariants.
  pub fn update(&mut self) {
    while let Some(dep_id) = self.work.pop() {
      // uniqueness
      let ty = self.canonalize(self.ty_map[dep_id.0].clone());
      if let Some(class_id) = self.id_map.get(&ty) {
        self.cong.insert(dep_id);
        self.union(
          // congruence
          *class_id, dep_id,
        );
      }
    }

    for class in self.qspace.values_mut() {
      class.nodes.retain(|node| {
        !self.cong.contains(node) // pdeps(cong) ∉ class
      });
    }

    self.cong.clear();
    for (id, class) in self.qspace.iter() {
      for node in class.nodes.iter() {
        assert_eq!(
          // ∀ node ∈ class, parent(node) = parent(class)
          self.search(*node),
          self.search(*id)
        );
      }
    }
  }

  /// Instantiate an expression from IR.
  pub fn inst(
    // cur_id < n
    &self,
    cur_id: HirId,
  ) -> T {
    assert!(cur_id.0 < self.parent.len());
    T::map(self[cur_id].clone(), |sub| self.inst(sub)).into()
  }

  /// Lower an expression into IR.
  pub fn lower(
    // ∀ expr: id
    &mut self,
    expr: T,
  ) -> HirId {
    let hir_ty = T::map(expr.into(), |sub| self.lower(sub));
    self.insert(
      hir_ty, // ty ∈ graph
    )
  }

  /// Insert an expression and return its unique Id.
  pub(crate) fn insert(
    // ty ∈ graph: id
    // ty ∉ graph: n + 1
    &mut self,
    ty: K,
  ) -> HirId {
    let ty = self.canonalize(ty);
    if let Some(cur_id) = self.id_map.get(&ty) {
      self.find(*cur_id)
    } else {
      self.new_id(ty)
    }
  }

  fn new_id(
    // ty ∉ graph
    &mut self,
    ty: K,
  ) -> HirId {
    let next_id = HirId(self.ty_map.len());
    // parent(n + 1) = n + 1
    self.parent.push(next_id);
    // ty_map = { 0 -> x_0, ... , n -> x_n, n + 1 -> x_ }
    self.ty_map.push(ty.clone());
    // id_map = { x_0 -> 0, ... , x_n -> n, x_ -> n + 1 }
    self.id_map.insert(ty.clone(), next_id);
    // qspace = { 0 ⊃ { 0, ..., n } }
    self.qspace.insert(
      next_id,
      HirClass {
        nodes: vec![next_id],
        pdeps: vec![],
      },
    );

    T::map(ty, |node| {
      let node = self.find(node);
      if let Some(class) = self.qspace.get_mut(&node) {
        class.pdeps.push(next_id)
      }
    });

    next_id
  }

  fn canonalize(
    // parent(ty) ∈ graph
    &mut self,
    ty: K,
  ) -> K {
    T::map(ty, |node| self.find(node))
  }
}

impl<T, K> Index<HirId> for Graph<T, K> {
  /// Index on a [`HirId`] type.
  type Output = K;

  fn index(&self, id: HirId) -> &Self::Output {
    &self.ty_map[id.0]
  }
}

impl fmt::Debug for HirId {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "`{}`", self.0)
  }
}

impl<T, K> fmt::Display for Graph<T, K>
where
  K: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (id, class) in &self.qspace {
      writeln!(f, "├─ ⊆ {{{id:?}}}: {:?}", self.ty_map[id.0])?;
      for eq_id in &class.nodes {
        if eq_id != id {
          writeln!(f, "│  ├─ {eq_id:?}: {:?}", self.ty_map[eq_id.0])?;
        }
      }
    }

    // stats
    write!(
      f,
      "\
      {} (total) expressions \n\
      {} classes     \n",
      self.ty_map.len(),
      self.qspace.len(),
    )?;
    Ok(())
  }
}
