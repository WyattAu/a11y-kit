//! The semantic node model (a11y-kit, ADR-0004 §2): role, accessible name,
//! states, and relations — platform-free, so it can be audited headlessly
//! in CI and lowered into AccessKit (or anything else) at the edge.

use core::fmt;

/// A stable node identity within one tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// The semantic roles the estate's controls map onto. Deliberately a
/// toolkit-neutral subset of the ARIA/accesskit role vocabulary — the
/// lowering adapters pick the framework's spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum A11yRole {
    /// Top-level window.
    Window,
    /// Generic container.
    Group,
    /// Section heading.
    Heading,
    /// Static text.
    Paragraph,
    /// Push button.
    Button,
    /// Toggle / tri-state checkbox.
    Checkbox,
    /// Single-line text entry.
    Textbox,
    /// Multi-line text entry.
    MultilineTextbox,
    /// Numeric slider.
    Slider,
    /// Dropdown / select.
    ComboBox,
    /// List container.
    List,
    /// List item.
    ListItem,
    /// Tab strip.
    TabList,
    /// One tab.
    Tab,
    /// Data table.
    Table,
    /// Table row.
    Row,
    /// Table cell.
    Cell,
    /// Toolbar.
    Toolbar,
    /// Menu container.
    Menu,
    /// Menu entry.
    MenuItem,
    /// Status/alert region (live region).
    StatusBar,
    /// Dialog.
    Dialog,
    /// Blocking alert.
    Alert,
    /// Progress indicator.
    ProgressBar,
    /// Link.
    Link,
    /// Separator.
    Separator,
    /// Image with alternatives.
    Image,
    /// Canvas/surface region.
    Canvas,
}

/// True for roles a keyboard user must be able to reach and operate —
/// roles whose accessible name the audit requires.
#[must_use]
pub const fn is_interactive(role: A11yRole) -> bool {
    matches!(
        role,
        A11yRole::Button
            | A11yRole::Checkbox
            | A11yRole::Textbox
            | A11yRole::MultilineTextbox
            | A11yRole::Slider
            | A11yRole::ComboBox
            | A11yRole::ListItem
            | A11yRole::Tab
            | A11yRole::MenuItem
            | A11yRole::Link
    )
}

/// Tri-state checkbox value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    /// Not checked.
    Unchecked,
    /// Checked.
    Checked,
    /// Indeterminate ("mixed").
    Mixed,
}

/// Node state flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct States {
    /// Disabled.
    pub disabled: bool,
    /// Focused (at most one per tree — audited).
    pub focused: bool,
    /// Selected.
    pub selected: bool,
    /// Expanded (collapsible containers).
    pub expanded: bool,
    /// Required (form input).
    pub required: bool,
    /// Invalid (form input with an error).
    pub invalid: bool,
    /// Hidden from the a11y tree (still present for focus management).
    pub hidden: bool,
    /// Modal (blocks sibling interaction).
    pub modal: bool,
    /// Checkbox value.
    pub checked: Option<TriState>,
    /// Busy: the value is transient or not yet settled.
    ///
    /// Two callers need this and neither had a way to say it (found by
    /// composing `input-kit` against this crate, 2026-10-18):
    ///
    /// - **IME composition.** A textbox whose value is a half-formed syllable
    ///   must not have that value announced. `aria-busy` is the signal, and
    ///   `input-kit`'s `Composer::is_composing` is what drives it.
    /// - **Async loading.** A textbox whose value is still being fetched should
    ///   not be announced as empty.
    ///
    /// `busy` is distinct from `disabled` and `invalid`: the control is live
    /// and valid, only its value is not final.
    pub busy: bool,
}

/// A relation between nodes (the edges a screen reader traverses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// This node controls the target (e.g. a toggle controls a panel).
    Controls(NodeId),
    /// The target describes this node (an error message).
    DescribedBy(NodeId),
    /// The target names this node.
    LabelledBy(NodeId),
    /// The focused control's current suggestion (combobox autocomplete).
    ActiveDescendant(NodeId),
}

/// One semantic node.
#[derive(Debug, Clone, PartialEq)]
pub struct A11yNode {
    /// Identity.
    pub id: NodeId,
    /// Semantic role.
    pub role: A11yRole,
    /// Accessible name (required for interactive roles).
    pub name: String,
    /// Longer description.
    pub description: Option<String>,
    /// State flags.
    pub states: States,
    /// Structural parent (None for a root).
    pub parent: Option<NodeId>,
    /// Non-structural relations.
    pub relations: Vec<Relation>,
}

impl A11yNode {
    /// A node with the common fields; states/relations default empty.
    #[must_use]
    pub fn new(id: NodeId, role: A11yRole, name: impl Into<String>) -> Self {
        Self {
            id,
            role,
            name: name.into(),
            description: None,
            states: States::default(),
            parent: None,
            relations: Vec::new(),
        }
    }

    /// Attach a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set states.
    #[must_use]
    pub fn with_states(mut self, states: States) -> Self {
        self.states = states;
        self
    }

    /// Set the structural parent.
    #[must_use]
    pub fn with_parent(mut self, parent: NodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Add a relation.
    pub fn with_relation(mut self, relation: Relation) -> Self {
        self.relations.push(relation);
        self
    }
}
