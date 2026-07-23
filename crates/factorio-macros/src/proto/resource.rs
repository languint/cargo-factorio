use super::define::define_proto;

define_proto! {
    fn: resource,
    ty: ResourceEntity,
    names: "Resources",
    register: "register_resources",
    fields: {
        name: str,
        icon: opt_icon,
        subgroup: opt_str,
        order: opt_str,
        flags: opt_flags,
    }
}
