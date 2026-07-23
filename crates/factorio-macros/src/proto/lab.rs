use super::define::define_proto;

define_proto! {
    fn: lab,
    ty: Lab,
    names: "Labs",
    register: "register_labs",
    fields: {
        name: str,
        energy_usage: str,
        energy_type: str,
        inputs: str_list,
        usage_priority: opt_str,
        icon: opt_icon,
        module_slots: opt_i64,
        subgroup: opt_str,
        order: opt_str,
        flags: opt_flags,
        max_health: opt_f64,
    },
    emit: {
        name,
        energy_usage,
        energy_source: energy(energy_type, usage_priority),
        inputs,
        icon,
        module_slots,
        subgroup,
        order,
        flags,
        max_health,
    }
}
