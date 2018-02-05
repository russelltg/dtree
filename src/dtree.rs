use std::collections::HashMap;

/// A mapping. Always stored in node
pub struct Mapping {
    /// The list of triggers that trigger this mapping
    pub triggers: Vec<String>,
    /// The description of the mapping
    pub description: String,
    /// The name of the node that it maps to. For any
    /// parsed tree, it is garunteed to be valid (or the parse will fail)
    pub to: String,
}

/// A section
pub struct Section {
    /// The name of this section.
    /// Should be an identifier (as defined in spec.md)
    pub name: String,

    /// The description of the section
    pub description: String,

    /// All the mappings
    pub mappings: Vec<Mapping>,
}

/// The full tree
pub struct Tree {
    /// All the sections
    pub sections: HashMap<String, Section>,
}

impl Mapping {

    /// Check if a mapping contains a given trigger
    pub fn has_trigger(&self, trigger: &str) -> bool {
        for t in &self.triggers {
            if t == trigger {
                return true;
            }
        }
        return false;
    }
}

impl Section {

    /// Get the destination node for a given trigger
    pub fn mapping<'a>(&'a self, trigger: &str) -> Option<&'a str> {
        for m in &self.mappings {
            if m.has_trigger(trigger) {
                return Some(&m.to);
            }
        }
        return None;
    }
}
