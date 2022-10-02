use std::{path::Path, process::Command};

use anyhow::{Context, Error, Result};

fn compile_shader(shader_file_path: &Path) -> Result<()> {
    let parent = shader_file_path.parent().with_context(|| {
        format!(
            "unable to get parent dir for shader at {:?}",
            shader_file_path
        )
    })?;
    let shader_file_name = shader_file_path
        .file_name()
        .with_context(|| {
            format!(
                "Unable to get file name for shader at path {:#?}",
                shader_file_path,
            )
        })?
        .to_str()
        .with_context(|| {
            format!(
                "Unable to get str representation of file name at path {:#?}",
                shader_file_path
            )
        })?;
    let output_file_name = format!("{}.spv", shader_file_name);

    let output_path = parent
        .join(Path::new(&output_file_name))
        .to_str()
        .unwrap()
        .to_owned();

    let output = Command::new("glslc")
        .arg(shader_file_path.to_str().unwrap())
        .arg("-o")
        .arg(&output_path)
        .arg("--target-env=vulkan1.3")
        .output()
        .unwrap();

    if !output.status.success() {
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        eprintln!("{}", stdout);
        eprintln!("{}", stderr);
        return Err(Error::msg(format!(
            "Error running glslc for shader at {:#?}",
            shader_file_path,
        )));
    } else {
        let shader_path_str = shader_file_path.to_str().unwrap();
        println!("cargo:warning={} -> {}", shader_path_str, output_path);
        println!("cargo:rerun-if-changed={}", shader_path_str);
    }

    Ok(())
}

fn main() -> Result<()> {
    for path_entry in glob::glob("./**/*.vert")? {
        compile_shader(path_entry?.as_path())?;
    }
    for path_entry in glob::glob("./**/*.frag")? {
        compile_shader(path_entry?.as_path())?;
    }
    for path_entry in glob::glob("./**/*.comp")? {
        compile_shader(path_entry?.as_path())?;
    }
    Ok(())
}
