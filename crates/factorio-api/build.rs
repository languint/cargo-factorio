use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let out = Path::new(&out_dir);

    let generated = factorio_api_gen::generate_from_bundled_api()?;
    factorio_api_gen::write_generated_api(out, &generated)?;

    let prototypes = factorio_api_gen::generate_prototypes_from_bundled()?;
    factorio_api_gen::write_generated_prototypes(out, &prototypes)?;

    println!("cargo:rerun-if-changed=../factorio-api-gen/api/runtime-api.json");
    println!("cargo:rerun-if-changed=../factorio-api-gen/api/prototype-api.json");
    println!("cargo:rerun-if-changed=../factorio-api-gen/src");
    Ok(())
}
