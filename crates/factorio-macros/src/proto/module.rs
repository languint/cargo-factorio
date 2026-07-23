use super::define::define_proto;

define_proto! {
    fn: module_proto,
    ty: Module,
    names: "Modules",
    register: "register_modules",
    fields: {
        name: str,
        stack_size: i64,
        category: str,
        tier: i64,
        icon: opt_icon,
        subgroup: opt_str,
        order: opt_str,
    }
}
