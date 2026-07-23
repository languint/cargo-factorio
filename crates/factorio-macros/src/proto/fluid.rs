use super::define::define_proto;

define_proto! {
    fn: fluid,
    ty: Fluid,
    names: "Fluids",
    register: "register_fluids",
    fields: {
        name: str,
        icon: req_icon,
        default_temperature: f64,
        base_color: color,
        flow_color: color,
        icon_size: opt_i64,
        subgroup: opt_str,
        order: opt_str,
        hidden: opt_bool,
    }
}
