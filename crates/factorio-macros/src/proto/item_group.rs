use super::define::define_proto;

define_proto! {
    fn: item_group,
    ty: ItemGroup,
    names: "ItemGroups",
    register: "register_item_groups",
    fields: {
        name: str,
        icon: opt_icon,
        order: opt_str,
        order_in_recipe: opt_str,
    }
}
