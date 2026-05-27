/// A demo for Boolean algebra.
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use given::*;
use given_macros::expr_impl;

#[expr_impl(Bool)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Expr<Node> {
  /// A boolean value (i.e., true or false)
  Value(bool),
  /// A variable.
  Var(Rc<str>),

  /// A negation `¬`.
  Not(Node),
  /// A conjunction `∧`.
  And(Node, Node),
  /// A disjunction `∨`.
  Or(Node, Node),
}

impl Bool {
  /// Create a new boolean value.
  pub fn value(value: bool) -> Self {
    Self::from(Expr::Value(value))
  }

  /// Create a new variable.
  pub fn variable(name: &str) -> Self {
    Self::from(Expr::Var(Rc::from(name)))
  }

  /// Generate ¬`self`.
  pub fn not(self) -> Self {
    Self::from(Expr::Not(self))
  }

  /// Generate `self` ∧ `o`.
  pub fn and(self, o: Self) -> Self {
    Self::from(Expr::And(self, o))
  }

  /// Generate `self` ∨ `o`.
  pub fn or(self, o: Self) -> Self {
    Self::from(Expr::Or(self, o))
  }
}

impl Bool {
  pub fn eval(
    // Var -> Value
    &self,
    env: &HashMap<Rc<str>, bool>,
  ) -> bool {
    use Expr::*;
    match self.as_ref() {
      Value(value) => *value,
      Var(ident) => env[ident],

      Not(arg) => !arg.eval(env),
      And(lhs, rhs) => lhs.eval(env) & rhs.eval(env),
      Or(lhs, rhs) => lhs.eval(env) | rhs.eval(env),
    }
  }

  fn fmt_with_prec(
    &self,
    f: &mut fmt::Formatter<'_>,
    // Format using precedence (default: 0)
    parent_prec: u8,
  ) -> fmt::Result {
    use Expr::*;

    fn precedence(expr: &Expr<Bool>) -> u8 {
      match expr {
        Or(_, _) => 1,
        And(_, _) => 2,
        Not(_) => 3,
        Value(_) | Var(_) => 4,
      }
    }

    let cur_prec = precedence(self.as_ref());
    let req_pars = cur_prec < parent_prec;
    if req_pars {
      write!(f, "(")?;
    }

    match self.as_ref() {
      Value(value) => {
        write!(f, "{value}")?;
      }

      Var(name) => {
        write!(f, "{name}")?;
      }

      Not(arg) => {
        write!(f, "¬")?;
        arg.fmt_with_prec(f, cur_prec)?;
      }

      And(lhs, rhs) => {
        lhs.fmt_with_prec(f, cur_prec)?;
        write!(f, " ∧ ")?;
        rhs.fmt_with_prec(f, cur_prec)?;
      }

      Or(lhs, rhs) => {
        lhs.fmt_with_prec(f, cur_prec)?;
        write!(f, " ∨ ")?;
        rhs.fmt_with_prec(f, cur_prec)?;
      }
    }

    if req_pars {
      write!(f, ")")?;
    }

    Ok(())
  }
}

impl Type for Bool {
  type Term<Node> = Expr<Node>;

  fn map<F, T>(
    // term -> f(term)
    term: Self::Term<F>,
    mut f: impl FnMut(F) -> T,
  ) -> Self::Term<T> {
    use Expr::*;
    match term {
      Value(value) => Value(value),
      Var(ident) => Var(ident),

      Not(arg) => Not(f(arg)),
      And(lhs, rhs) => And(f(lhs), f(rhs)),
      Or(lhs, rhs) => Or(f(lhs), f(rhs)),
    }
  }
}

impl fmt::Display for Bool {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    self.fmt_with_prec(f, 0)
  }
}

fn main() {
  let x = Bool::variable("x");
  let e1 = x.clone().or(x.not());
  println!("e1 = {e1}.");
  let e2 = e1.clone().not().and(e1.clone().not()).not();
  println!("e2 = {e2}.");
  let e3 = Bool::value(true);
  let mut graph = Graph::new();
  let id_e1 = graph.lower(e1.clone());
  let id_e2 = graph.lower(e2.clone());
  let id_e3 = graph.lower(e3.clone());

  println!("We try to prove that {e2} ↔ {e3} using equivalences.");
  println!("{e2} ↔ {e3}: {}.", graph.find(id_e2) == graph.find(id_e3));

  println!("Pass 1:");
  println!("Applying union {e1} → {e3}.");
  graph.union(id_e1, id_e3);
  let e4 = e1.clone().or(e1.clone());
  let id_e4 = graph.lower(e4.clone());
  println!("Applying union {e2} → {e4}.");
  graph.union(id_e2, id_e4);
  graph.update();
  println!("{e2} ↔ {e3}: {}.", graph.find(id_e2) == graph.find(id_e3));

  println!("Pass 2:");
  println!("Applying union {e2} → {e1}.");
  graph.union(id_e2, id_e1);
  graph.update();
  println!("{e2} ↔ {e3}: {}.", graph.find(id_e2) == graph.find(id_e3));

  let expr = graph.inst(id_e4);
  println!("Validate that e4 = {expr} is always true:");
  println!(
    "e4 {{x = false}}: {}.",
    expr.eval(&HashMap::from([(Rc::from("x"), false)]))
  );
  println!(
    "e4 {{x = true}}: {}.",
    expr.eval(&HashMap::from([(Rc::from("x"), true)]))
  );

  println!("Final equivalence graph representation:");
  println!("{graph}");
}
