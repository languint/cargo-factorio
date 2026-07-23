use super::define::define_proto;

define_proto! {
    fn: furnace,
    ty: Furnace,
    names: "Furnaces",
    register: "register_furnaces",
    fields: {
        name: str,
        crafting_speed: f64,
        crafting_categories: str_list,
        energy_usage: str,
        energy_type: str,
        result_inventory_size: i64,
        source_inventory_size: i64,
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
        crafting_speed,
        crafting_categories,
        energy_usage,
        energy_source: energy(energy_type, usage_priority),
        result_inventory_size,
        source_inventory_size,
        icon,
        module_slots,
        subgroup,
        order,
        flags,
        max_health,
    }
}
