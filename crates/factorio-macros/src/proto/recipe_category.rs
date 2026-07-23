use super::define::define_proto;

define_proto! {
    fn: recipe_category,
    ty: RecipeCategory,
    names: "RecipeCategories",
    register: "register_recipe_categories",
    fields: {
        name: str,
        order: opt_str,
        hidden: opt_bool,
    }
}
