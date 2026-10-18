# a11y-kit

The accessibility semantics layer for the estate — the keystone nobody in
the ecosystem funds (ADR-0004 §2; measured Oct 2026: AccessKit runs on one
maintainer's spare time, has no web adapter, and *no project ships an a11y CI
gate*).

```rust
use a11y_kit::{A11yNode, A11yRole, A11yTree, NodeId, Relation, States};

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
assert!(tree.audit().is_ok());
```

## The gate

`tree.audit()` enforces five rules headlessly in CI:

1. ids are unique;
2. parents and relation targets resolve;
3. no parent cycles;
4. every interactive role carries an accessible name;
5. at most one focused node, and it lives inside a `Window` subtree — the
   "focus is somewhere the screen reader can't see" bug.

Every failure names the node (`AuditError::MissingAccessibleName(NodeId(5))`),
because a gate that cannot be acted on is not a gate.

## Properties

- **L1 substrate** — dependency-free by default (`thiserror` only);
  AccessKit transport conversion lands behind the `accesskit` feature
  (Wave 2.1) so the audit layer stays headless and any CI can run it.
- Toolkits lower *into* this model; none of them own the gate.

Licensed under MIT OR Apache-2.0.
