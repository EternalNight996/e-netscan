#![allow(dead_code)]
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
#[cfg(windows)]
use winapi;
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        // #[cfg(not(debug_assertions))]
        // build_static()?;
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("static/icon.ico")
            .set_icon_with_id("static/icon.ico", "2")
            .set_output_directory("target/")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_CHINESE_SIMPLIFIED,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .compile()?;
    }
    Ok(())
}

/// building static resource
fn build_static() -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let target_dir = PathBuf::from(&current_dir).join("target").join("release");
    // if target dir is not exist then create all;
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)?;
    }
    for s in ["Install.bat", "Uninstall.bat", "static", "scripts"] {
        let src = PathBuf::from(&current_dir).join(s);
        let dst = target_dir.join(s);
        copy_dirs_file(&src, &dst)?;
    }
    Ok(())
}

/// walk up copy
fn copy_dirs_file(src: &Path, dst: &Path) -> io::Result<()> {
    if src.exists() {
        if src.is_dir() {
            if !dst.exists() {
                fs::create_dir_all(&dst)?;
            }
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let src_path = entry.path();
                let dst_path = PathBuf::from(dst).join(entry.file_name());
                if src_path.is_file() || src_path.is_symlink() {
                    println!("Trying copy from {:?} to {:?}", src_path, dst_path);
                    fs::copy(src_path, dst_path)?;
                } else if src_path.is_dir() {
                    println!("Goto {:?}", src_path);
                    copy_dirs_file(&src_path, &dst_path)?;
                } else {
                    println!("Unknown {:?} of type", src_path);
                }
            }
        } else if src.is_file() {
            fs::copy(src, dst)?;
        } else {
            println!("Unknown {:?} of type", src);
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{:?}", src),
        ));
    }
    Ok(())
}
