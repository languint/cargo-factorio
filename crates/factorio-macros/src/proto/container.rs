use super::define::define_proto;

define_proto! {
    fn: container,
    ty: Container,
    names: "Containers",
    register: "register_containers",
    fields: {
        name: str,
        inventory_size: i64,
        icon: opt_icon,
        icon_size: parse_i64,
        subgroup: opt_str,
        order: opt_str,
        flags: opt_flags,
        max_health: opt_f64,
    }
}
