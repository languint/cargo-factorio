use super::define::define_proto;

define_proto! {
    fn: autoplace_control,
    ty: AutoplaceControl,
    names: "AutoplaceControls",
    register: "register_autoplace_controls",
    fields: {
        name: str,
        category: str,
        order: opt_str,
        hidden: opt_bool,
    }
}
