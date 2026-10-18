//! a11y-kit — the accessibility semantics layer (ADR-0004 §2; the keystone
//! nobody in the ecosystem funds).
//!
//! A platform-free semantic node model — role, accessible name, states,
//! relations — plus a **tree audit that runs headlessly in CI**. Every
//! toolkit lowers into this; none of them ship the gate.
//!
//! ```
//! use a11y_kit::{A11yNode, A11yRole, A11yTree, NodeId, Relation, States};
//!
//! let mut tree = A11yTree::new();
//! let root = tree.push(A11yNode::new(NodeId(0), A11yRole::Window, "Chisel Docs"));
//! let panel = tree.push(
//!     A11yNode::new(NodeId(1), A11yRole::Group, "Tool palette")
//!         .with_parent(root)
//!         .with_states(States { focused: true, ..States::default() }),
//! );
//! let save = tree.push(
//!     A11yNode::new(NodeId(2), A11yRole::Button, "Save document")
//!         .with_parent(panel)
//!         .with_relation(Relation::LabelledBy(panel)),
//! );
//! # let _ = save;
//! assert!(tree.audit().is_ok(), "a well-formed tree passes");
//! ```
//!
//! # The rules the audit enforces
//!
//! 1. ids are unique;
//! 2. parents/relations resolve;
//! 3. no parent cycles;
//! 4. every interactive role carries an accessible name;
//! 5. at most one focused node, and it must live inside a `Window` subtree
//!    (a focusable node floating as its own root is invisible to the
//!    screen reader).
//!
//! # AccessKit
//!
//! Conversion into AccessKit's transport types lands behind the `accesskit`
//! feature (Wave 2.1, first toolkit adapter). The audit layer stays
//! dependency-free so any CI can run it — including this repo's.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod node;
mod tree;

pub use error::AuditError;
pub use node::{is_interactive, A11yNode, A11yRole, NodeId, Relation, States, TriState};
pub use tree::{A11yTree, TreeAudit};

/// The crate's version, for callers that report it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
