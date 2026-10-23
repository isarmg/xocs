use std::{path::Path, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap();
    for sample in 0..6 {
        let start = Instant::now();
        let value = blog_assets::inventory::snapshot(Path::new(&path))?;
        println!("sample={sample} elapsed_seconds={:.6} foundation_files={} package_records={} digest={}", start.elapsed().as_secs_f64(), value["foundation_inputs"].as_object().unwrap().len(), value["patched_inputs"].as_object().unwrap().len(), blog_assets::inventory::sha(value.to_string().as_bytes()));
    }
    Ok(())
}
