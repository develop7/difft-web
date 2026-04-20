#[cfg(target_os = "wasi")]
fn main() {}

#[cfg(not(target_os = "wasi"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let diffs = difft_web::generate_demo_diffs()?;
    let output = serde_json::to_string_pretty(&diffs)?;
    std::fs::create_dir_all("docs")?;
    std::fs::write("docs/examples.json", output)?;
    println!("wrote docs/examples.json");
    Ok(())
}
