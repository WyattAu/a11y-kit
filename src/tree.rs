//! The tree and the audit — the CI gate nobody in the ecosystem has
//! (AccessKit ships no cross-screen-reader harness; every toolkit's
//! a11y CI is hand-rolled per project).

use crate::error::AuditError;
use crate::node::{is_interactive, A11yNode, NodeId};

/// What an audit measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeAudit {
    /// Nodes in the tree.
    pub nodes: usize,
    /// Roots (parentless nodes).
    pub roots: usize,
    /// Interactive roles checked for names.
    pub interactive: usize,
    /// Relation edges validated.
    pub relations: usize,
}

/// An insertion-ordered tree of semantic nodes.
#[derive(Debug, Clone, Default)]
pub struct A11yTree {
    nodes: Vec<A11yNode>,
}

impl A11yTree {
    /// An empty tree.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a node; ids must be unique and ascending is conventional but
    /// not required. Returns the node's id for chaining.
    pub fn insert(&mut self, node: A11yNode) -> NodeId {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// Insert with the next free id.
    pub fn push(&mut self, mut node: A11yNode) -> NodeId {
        node.id = NodeId(self.nodes.len() as u32);
        self.insert(node)
    }

    /// Look up a node.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&A11yNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// All nodes, in insertion order.
    pub fn nodes(&self) -> &[A11yNode] {
        &self.nodes
    }

    /// Roots (nodes without a parent).
    #[must_use]
    pub fn roots(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.parent.is_none())
            .map(|n| n.id)
            .collect()
    }

    /// The focused node, if exactly one claims focus.
    #[must_use]
    pub fn focused(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.states.focused)
            .map(|n| n.id)
            .collect()
    }

    /// Audit the tree. Every rule is a CI gate:
    ///
    /// 1. unique ids;
    /// 2. parents and relation targets exist;
    /// 3. no structural cycles;
    /// 4. interactive roles carry an accessible name;
    /// 5. at most one focused node, and it is reachable from a root.
    pub fn audit(&self) -> Result<TreeAudit, AuditError> {
        // (1) unique ids
        for (i, node) in self.nodes.iter().enumerate() {
            if self.nodes.iter().skip(i + 1).any(|n| n.id == node.id) {
                return Err(AuditError::DuplicateId(node.id));
            }
        }

        // (2) dangling references
        for node in &self.nodes {
            if let Some(parent) = node.parent {
                if self.get(parent).is_none() {
                    return Err(AuditError::DanglingReference {
                        node: node.id,
                        target: parent,
                    });
                }
            }
            for relation in &node.relations {
                let target = match relation {
                    crate::node::Relation::Controls(id)
                    | crate::node::Relation::DescribedBy(id)
                    | crate::node::Relation::LabelledBy(id)
                    | crate::node::Relation::ActiveDescendant(id) => *id,
                };
                if self.get(target).is_none() {
                    return Err(AuditError::DanglingReference {
                        node: node.id,
                        target,
                    });
                }
            }
        }

        // (3) structural cycles: walk each node up to a root
        for node in &self.nodes {
            let mut seen = vec![node.id];
            let mut current = node.parent;
            while let Some(id) = current {
                if seen.contains(&id) {
                    return Err(AuditError::Cycle { node: node.id });
                }
                seen.push(id);
                current = self.get(id).and_then(|n| n.parent);
            }
        }

        // (4) interactive roles must be named
        let mut interactive = 0usize;
        for node in &self.nodes {
            if is_interactive(node.role) {
                interactive += 1;
                if node.name.trim().is_empty() {
                    return Err(AuditError::MissingAccessibleName(node.id));
                }
            }
        }

        // (5) focus discipline: at most one focused node, and it must live
        // inside a Window subtree — a focusable node floating as its own
        // root is the classic "the screen reader can't see what has focus"
        // bug.
        let focused = self.focused();
        if focused.len() > 1 {
            return Err(AuditError::MultipleFocus(focused));
        }
        if let Some(id) = focused.first() {
            let in_window = self
                .nodes
                .iter()
                .filter(|n| n.role == crate::node::A11yRole::Window)
                .flat_map(|w| self.descendants_of(w.id))
                .any(|n| n == *id);
            if !in_window {
                return Err(AuditError::UnreachableFocus(*id));
            }
        }

        let relations = self.nodes.iter().map(|n| n.relations.len()).sum();
        Ok(TreeAudit {
            nodes: self.nodes.len(),
            roots: self.roots().len(),
            interactive,
            relations,
        })
    }

    /// Every node reachable by walking parents down from the roots.
    #[must_use]
    pub fn reachable_from_roots(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = self.roots();
        while let Some(id) = stack.pop() {
            out.push(id);
            for child in self.children_of(id) {
                stack.push(child);
            }
        }
        out
    }

    /// Direct children of `id`.
    #[must_use]
    pub fn children_of(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.parent == Some(id))
            .map(|n| n.id)
            .collect()
    }

    /// `id` and every node beneath it (depth-first, parents first).
    #[must_use]
    pub fn descendants_of(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = vec![id];
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            for child in self.children_of(current) {
                out.push(child);
                stack.push(child);
            }
        }
        out
    }
}
