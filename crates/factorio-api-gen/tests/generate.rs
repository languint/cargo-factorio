use factorio_api_gen::{generate_from_bundled_api, generate_runtime_api, parse_runtime_api};

#[test]
fn bundled_runtime_api_parses() {
    let api = generate_from_bundled_api().expect("bundled runtime-api.json should parse");
    assert!(!api.events.is_empty());
    assert!(api.event_map.contains("event_type_to_name"));
    assert!(api.classes.contains("LuaGameScript"));
    assert!(api.globals.contains("pub static game"));
}

#[test]
fn maps_events_to_rust_names() {
    let api = parse_runtime_api(factorio_api_gen::bundled_runtime_api_json())
        .expect("bundled runtime-api.json should parse");
    let generated = generate_runtime_api(&api);

    assert!(generated.events.contains("OnSingleplayerInit"));
    assert!(generated.event_map.contains("on_singleplayer_init"));
    assert!(generated.event_lookup.contains("OnSingleplayerInit"));
}
