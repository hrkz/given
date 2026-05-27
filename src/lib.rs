//!
//! Given and API reference documentation.
//!

// Session context
mod ctx;
// Equivalence graph
mod eq_graph;
// Pattern matching
mod pat;
// Type lowering
mod ty;

pub use crate::{
  // Main imports
  eq_graph::Graph,
  ty::Type,
};
