use super::define::define_proto;

define_proto! {
    fn: item_subgroup,
    ty: ItemSubgroup,
    names: "ItemSubgroups",
    register: "register_item_subgroups",
    fields: {
        name: str,
        group: str,
        order: opt_str,
    }
}
