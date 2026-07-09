fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let generated = factorio_api_gen::generate_from_bundled_api()?;

    factorio_api_gen::write_macro_event_lookup(std::path::Path::new(&out_dir), &generated)?;

    println!("cargo:rerun-if-changed=../factorio-api-gen/api/runtime-api.json");
    Ok(())
}
