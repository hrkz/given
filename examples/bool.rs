/// A demo for Boolean algebra.
use std::collections::HashMap;
use std::fmt;
use std::ops;
use std::rc::Rc;

use given::*;

/// "Flat" AST
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Bool<Node> {
  /// A boolean literal (true / false).
  Lit(bool),
  /// A boolean variable
  ///
  /// Denoted e.g., ?x by convention.
  Var(Rc<str>),
  /// A logical negation (`¬a`).
  Not(Node),
  /// A logical conjunction (`a ∧ b`).
  And(Node, Node),
  /// A logical disjunction (`a ∨ b`).
  Or(Node, Node),
}

/// Ast structure helper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Ast(Box<Expr>);
/// "Full" AST
type Expr = Bool<Ast>;

impl Expr {
  /// Literal `value`.
  pub fn lit(value: bool) -> Self {
    Self::Lit(value)
  }

  /// Variable `symbol`.
  pub fn var(symbol: &str) -> Self {
    Self::Var(Rc::from(symbol))
  }

  /// Logical negation `¬a`.
  pub fn not(self) -> Self {
    Self::Not(Ast(Box::new(self)))
  }

  /// Logical conjunction `a ∧ b`.
  pub fn and(self, o: Self) -> Self {
    Self::And(Ast(Box::new(self)), Ast(Box::new(o)))
  }

  /// Logical disjunction `a ∨ b`.
  pub fn or(self, o: Self) -> Self {
    Self::Or(Ast(Box::new(self)), Ast(Box::new(o)))
  }

  /// Material implication:
  /// `a → b = ¬a ∨ b`
  pub fn imply(self, o: Self) -> Self {
    self.not().or(o)
  }

  /// Material equivalence:
  /// `a ↔ b = (a ∧ b) ∨ (¬a ∧ ¬b)`
  pub fn equiv(self, o: Self) -> Self {
    self.clone().and(o.clone()).or(self.not().and(o.not()))
  }

  /// Exclusive or:
  /// `a ⊕ b = (a ∨ b) ∧ ¬(a ∧ b)`
  pub fn xor(self, o: Self) -> Self {
    self.clone().or(o.clone()).and(self.and(o).not())
  }
}

impl Type for Expr {
  type Term<Node> = Bool<Node>;

  fn map<F, T>(
    // term -> f(term)
    term: Self::Term<F>,
    mut f: impl FnMut(F) -> T,
  ) -> Self::Term<T> {
    use Bool::*;
    match term {
      Lit(value) => Lit(value),
      Var(symbol) => Var(symbol),

      Not(arg) => Not(f(arg)),
      And(lhs, rhs) => And(f(lhs), f(rhs)),
      Or(lhs, rhs) => Or(f(lhs), f(rhs)),
    }
  }

  fn lower(
    // expr -> term
    self,
  ) -> Self::Term<Self> {
    return Expr::map(self, |f| *f.0);
  }

  fn inst(
    // term -> expr
    term: Self::Term<Self>,
  ) -> Self {
    return Expr::map(term, |f| Ast(Box::new(f)));
  }
}

impl Expr {
  pub fn eval(
    // Var -> Lit
    &self,
    env: &HashMap<Rc<str>, bool>,
  ) -> bool {
    use Bool::*;
    match self {
      Lit(lit) => *lit,
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
    use Bool::*;

    fn precedence(kind: &Expr) -> u8 {
      match kind {
        And(..) => 1,
        Or(..) => 2,
        Not(_) => 3,
        Lit(_) | Var(_) => 4,
      }
    }

    let cur_prec = precedence(&self);
    let req_pars = cur_prec < parent_prec;
    if req_pars {
      write!(f, "(")?;
    }

    match &self {
      Lit(lit) => {
        write!(f, "{lit}")?;
      }

      Var(ident) => {
        write!(f, "?{ident}")?;
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

impl ops::Deref for Ast {
  type Target = Expr;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    self.fmt_with_prec(f, 0)
  }
}

fn main() {
  let x = Expr::var("x");
  let e1 = x.clone().or(x.not());
  println!("e1 = {e1}");
  let e2 = e1.clone().not().and(e1.clone().not()).not();
  println!("e2 = {e2}");
  let e3 = Expr::lit(true);
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
