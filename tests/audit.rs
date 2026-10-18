//! The gate itself: every rule, with the node it names. Test targets lift
//! the deny-level lints at file scope, per the estate pattern.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use a11y_kit::{
    A11yNode, A11yRole, A11yTree, AuditError, NodeId, Relation, States, TriState,
};

fn good_tree() -> A11yTree {
    let mut tree = A11yTree::new();
    let window = tree.push(A11yNode::new(NodeId(0), A11yRole::Window, "Chisel Docs"));
    let toolbar = tree.push(
        A11yNode::new(NodeId(1), A11yRole::Toolbar, "Tool palette")
            .with_parent(window)
            .with_states(States { focused: true, ..States::default() }),
    );
    tree.push(
        A11yNode::new(NodeId(2), A11yRole::Button, "Save document")
            .with_parent(toolbar)
            .with_relation(Relation::LabelledBy(toolbar)),
    );
    tree.push(
        A11yNode::new(NodeId(3), A11yRole::Checkbox, "Grid snap")
            .with_parent(toolbar)
            .with_states(States {
                checked: Some(TriState::Mixed),
                ..States::default()
            }),
    );
    tree.push(A11yNode::new(NodeId(4), A11yRole::StatusBar, "Ready").with_parent(window));
    tree
}

#[test]
fn a_well_formed_tree_passes_with_counts() {
    let audit = good_tree().audit().expect("the estate's own example tree is well-formed");
    assert_eq!(audit.nodes, 5);
    assert_eq!(audit.roots, 1);
    assert_eq!(audit.interactive, 2);
    assert_eq!(audit.relations, 1);
}

#[test]
fn interactive_roles_must_be_named() {
    let mut tree = good_tree();
    tree.push(A11yNode::new(NodeId(5), A11yRole::Button, "   ").with_parent(NodeId(0)));    assert_eq!(
        tree.audit(),
        Err(AuditError::MissingAccessibleName(NodeId(5))),
        "a whitespace name is no name"
    );
}

#[test]
fn decorative_roles_need_no_name() {
    let mut tree = good_tree();
    tree.push(A11yNode::new(NodeId(5), A11yRole::Separator, "").with_parent(NodeId(0)));
    assert!(tree.audit().is_ok());
}

#[test]
fn dangling_parents_and_relations_are_caught() {
    let mut tree = good_tree();
    tree.insert(A11yNode::new(NodeId(9), A11yRole::Group, "Orphan").with_parent(NodeId(42)));
    assert_eq!(
        tree.audit(),
        Err(AuditError::DanglingReference { node: NodeId(9), target: NodeId(42) })
    );

    let mut tree2 = good_tree();
    tree2.insert(
        A11yNode::new(NodeId(8), A11yRole::ComboBox, "Zoom")
            .with_parent(NodeId(0))
            .with_relation(Relation::Controls(NodeId(77))),
    );
    assert_eq!(
        tree2.audit(),
        Err(AuditError::DanglingReference { node: NodeId(8), target: NodeId(77) })
    );
}

#[test]
fn parent_cycles_are_caught() {
    let mut tree = good_tree();
    let a = tree.insert(A11yNode::new(NodeId(20), A11yRole::Group, "A").with_parent(NodeId(21)));
    tree.insert(A11yNode::new(NodeId(21), A11yRole::Group, "B").with_parent(a));
    assert_eq!(tree.audit(), Err(AuditError::Cycle { node: a }));
}

#[test]
fn duplicate_ids_are_caught() {
    let mut tree = good_tree();
    tree.insert(A11yNode::new(NodeId(2), A11yRole::Group, "Dup").with_parent(NodeId(0)));
    assert_eq!(tree.audit(), Err(AuditError::DuplicateId(NodeId(2))));
}

#[test]
fn focus_discipline_is_enforced() {
    let mut tree = good_tree();
    tree.insert(
        A11yNode::new(NodeId(6), A11yRole::Button, "Print")
            .with_parent(NodeId(0))
            .with_states(States { focused: true, ..States::default() }),
    );
    assert_eq!(tree.audit(), Err(AuditError::MultipleFocus(vec![NodeId(1), NodeId(6)])));
}

#[test]
fn a_focused_orphan_is_caught() {
    let mut tree = A11yTree::new();
    tree.push(A11yNode::new(NodeId(0), A11yRole::Window, "Root"));
    // A focusable node that is not parented under the root at all — the
    // classic "focused thing is invisible to the screen reader" bug.
    let orphan = tree.insert(
        A11yNode::new(NodeId(1), A11yRole::Textbox, "Search")
            .with_states(States { focused: true, ..States::default() }),
    );
    assert_eq!(tree.audit(), Err(AuditError::UnreachableFocus(orphan)));
}

#[test]
fn reachability_walks_from_the_roots() {
    let tree = good_tree();
    let reachable = tree.reachable_from_roots();
    assert_eq!(reachable.len(), 5, "every node in the estate example is reachable");
    assert!(reachable.contains(&NodeId(4)));
}

#[test]
fn the_audit_is_deterministic() {
    let tree = good_tree();
    assert_eq!(tree.audit(), tree.audit());
}
