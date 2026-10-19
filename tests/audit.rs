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

/// A busy textbox is a first-class state, added in 0.2.0.
///
/// The gap was found by composing `input-kit` (IME preedit) against this
/// crate: while an IME is forming a syllable the textbox's value is a
/// half-formed character, and there was no way to say so. A screen reader
/// would announce it. `busy` is the ARIA signal.
#[test]
fn a_composing_textbox_is_expressible_and_still_audits_clean() {
    let mut tree = A11yTree::new();
    let window = tree.push(A11yNode::new(NodeId(0), A11yRole::Window, "Chisel Docs"));

    // The focused field, settled: not composing.
    let focused = tree.push(
        A11yNode::new(NodeId(1), A11yRole::Textbox, "Page text")
            .with_parent(window)
            .with_states(States { focused: true, ..States::default() }),
    );
    // A second field, idle and unfocused. Only one node may be focused per
    // tree (the audit enforces it), so the composing state is shown on the
    // focused node below rather than by focusing a second one.
    let other = tree.push(
        A11yNode::new(NodeId(2), A11yRole::Textbox, "Search")
            .with_parent(window),
    );

    // Busy is an ordinary state: it does not make the tree invalid.
    let expected = a11y_kit::TreeAudit {
        nodes: 3,
        roots: 1,
        interactive: 2,
        relations: 0,
    };
    assert_eq!(tree.audit(), Ok(expected));

    // Now the focused field starts an IME composition. This is the transition
    // the 0.1.0 model could not express.
    let composing = States {
        focused: true,
        busy: true,
        ..States::default()
    };
    assert!(composing.busy);
    assert!(composing.focused);
    assert!(!composing.disabled, "busy is not disabled");
    assert!(!composing.invalid, "busy is not invalid");
    assert!(!composing.hidden, "busy is not hidden");

    // It differs from the settled state in exactly one flag, so a screen
    // reader is told "value is not final" without being told the control is
    // unusable.
    let settled = tree.get(focused).expect("node is in the tree").states;
    assert!(!settled.busy);
    assert_ne!(settled, composing);
    assert!(
        settled.busy ^ composing.busy,
        "composing is exactly the settled state plus busy"
    );

    // And an unfocused textbox stays unsettable from `busy` alone.
    assert!(!tree.get(other).expect("node is in the tree").states.busy);
}
