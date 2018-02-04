use std::collections::HashMap;

pub struct Mapping {
    pub description: String,
    pub to: String,
}

pub struct Node {
    pub name: String,
    pub description: String,
    pub mappings: HashMap<String, Mapping>,
}

pub struct Tree {
    pub nodes: HashMap<String, Node>,
}
