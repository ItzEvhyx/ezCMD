use crate::model::{Category, Command, Group, Section, Tool};

include!(concat!(env!("OUT_DIR"), "/content.rs"));

pub fn all_tools() -> impl Iterator<Item = &'static Tool> {
    CATEGORIES.iter().flat_map(|category| category.tools.iter())
}
