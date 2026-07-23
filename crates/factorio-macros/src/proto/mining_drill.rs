use super::define::define_proto;

define_proto! {
    fn: mining_drill,
    ty: MiningDrill,
    names: "MiningDrills",
    register: "register_mining_drills",
    fields: {
        name: str,
        mining_speed: f64,
        energy_usage: str,
        energy_type: str,
        resource_categories: str_list,
        resource_searching_radius: f64,
        usage_priority: opt_str,
        icon: opt_icon,
        module_slots: opt_i64,
        subgroup: opt_str,
        order: opt_str,
        flags: opt_flags,
    },
    emit: {
        name,
        mining_speed,
        energy_usage,
        energy_source: energy(energy_type, usage_priority),
        resource_categories,
        resource_searching_radius,
        icon,
        module_slots,
        subgroup,
        order,
        flags,
    }
}
