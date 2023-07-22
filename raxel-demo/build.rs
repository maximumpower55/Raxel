use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("PROFILE").unwrap();
    let out_dir = PathBuf::from(format!("../target/{}", out_dir));
    fs_extra::dir::copy(
        "./src/resources",
        &out_dir,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )?;
    Ok(())
}
