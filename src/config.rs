use std::{io, fs, path::PathBuf};

#[inline]
pub fn config_dir() -> PathBuf {
    let mut home = dirs::home_dir().unwrap();
    home.push(".config");
    home.push("fav");
    home
}

#[inline]
pub fn db_path() -> PathBuf {
    let mut p = config_dir();
    p.push("fav.db");
    p
}

pub fn init_config() -> io::Result<()> {
    fs::create_dir_all(config_dir())?;    
    Ok(())
}