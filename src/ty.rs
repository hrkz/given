/// A typed term-based AST to manipulate.
///
/// [`Type`] is the required trait that enables interactions with the (H)IR.
/// It is based on the definition of [`Type::Term`]s as generic types over node values.
/// A [`Type`] should be implemented on the recursive type that specifies
/// this generic [`Type::Term`] over a `Sized` type such as [`Box`].
pub trait Type: Sized {
  /// The generic term that defines a recursive type `Self`.
  type Term<Ty>;

  /// Apply a function to each `Ty` of a [`Self::Term`].
  fn map<F, T>(term: Self::Term<F>, f: impl FnMut(F) -> T) -> Self::Term<T>;

  /// Lower a [`Type`] AST into a [`Self::Term`].
  fn lower(self) -> Self::Term<Self>;
  /// Instantiate a [`Self::Term`] into a [`Type`] AST.
  fn inst(term: Self::Term<Self>) -> Self;

  fn is_leaf(&self) -> bool {
    todo!()
  }
}
