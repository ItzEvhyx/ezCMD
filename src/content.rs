use crate::model::{
    Category, Command, ContextPolicy, ContextSkipRule, Group, PackageManager, Recipe, RecipeOption,
    SafetyPolicy, Section, Tool,
};

include!(concat!(env!("OUT_DIR"), "/content.rs"));
include!(concat!(env!("OUT_DIR"), "/recipes.rs"));
include!(concat!(env!("OUT_DIR"), "/safety.rs"));
include!(concat!(env!("OUT_DIR"), "/context.rs"));

pub fn all_tools() -> impl Iterator<Item = &'static Tool> {
    CATEGORIES.iter().flat_map(|category| category.tools.iter())
}

pub fn all_recipes() -> &'static [Recipe] {
    RECIPES
}

pub fn safety_policy() -> &'static SafetyPolicy {
    &SAFETY_POLICY
}

pub fn context_policy() -> &'static ContextPolicy {
    &CONTEXT_POLICY
}
