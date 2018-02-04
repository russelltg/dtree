use std::collections::HashMap;

/// A mapping. Always stored in node
pub struct Mapping {
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
    pub mappings: HashMap<String, Mapping>,
}

/// The full tree
pub struct Tree {
    /// All the sections
    pub sections: HashMap<String, Section>,
}
