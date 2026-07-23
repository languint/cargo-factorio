use super::define::define_proto;

define_proto! {
    fn: transport_belt,
    ty: TransportBelt,
    names: "TransportBelts",
    register: "register_transport_belts",
    fields: {
        name: str,
        speed: f64,
        icon: opt_icon,
        subgroup: opt_str,
        order: opt_str,
        flags: opt_flags,
        max_health: opt_f64,
    }
}
