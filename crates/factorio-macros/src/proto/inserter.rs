use super::define::define_proto;

define_proto! {
    fn: inserter,
    ty: Inserter,
    names: "Inserters",
    register: "register_inserters",
    fields: {
        name: str,
        extension_speed: f64,
        rotation_speed: f64,
        energy_type: str,
        energy_usage: parse_str = "5kW",
        usage_priority: opt_str,
        icon: opt_icon,
        subgroup: opt_str,
        order: opt_str,
        flags: opt_flags,
        max_health: opt_f64,
    },
    emit: {
        name,
        extension_speed,
        rotation_speed,
        energy_source: energy(energy_type, usage_priority),
        icon,
        subgroup,
        order,
        flags,
        max_health,
    }
}
