//! Typed audit failures. Every variant names the node — a gate that says
//! "something is wrong with the a11y tree" is a gate nobody can fix.

use crate::node::NodeId;

/// Why an accessibility tree failed its audit.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuditError {
    /// Two nodes share an id.
    #[error("duplicate node id {0}")]
    DuplicateId(NodeId),
    /// A parent or relation points at a node that does not exist.
    #[error("node {node} references missing node {target}")]
    DanglingReference {
        /// The referencing node.
        node: NodeId,
        /// The missing node.
        target: NodeId,
    },
    /// The parent chain loops.
    #[error("node {node} is part of a parent cycle")]
    Cycle {
        /// A node on the cycle.
        node: NodeId,
    },
    /// An interactive role has no accessible name.
    #[error("interactive node {0} has no accessible name")]
    MissingAccessibleName(NodeId),
    /// More than one node claims focus.
    #[error("multiple nodes claim focus: {0:?}")]
    MultipleFocus(Vec<NodeId>),
    /// The focused node is not reachable from any root (an orphan subtree).
    #[error("focused node {0} is unreachable from any root")]
    UnreachableFocus(NodeId),
}
