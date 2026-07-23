use super::define::define_proto;

define_proto! {
    fn: item,
    ty: Item,
    names: "Items",
    register: "register",
    fields: {
        name: str,
        icon: req_icon,
        stack_size: i64,
        icon_size: opt_i64,
        subgroup: opt_str,
        order: opt_str,
    }
}
