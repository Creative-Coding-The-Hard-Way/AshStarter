use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Error, Result};

fn output_file_for_shader_file(shader_file_path: &Path) -> Result<PathBuf> {
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
    Ok(parent.join(Path::new(&output_file_name)))
}

fn needs_rebuild(shader_file_path: &Path, output_path: &Path) -> Result<bool> {
    if !output_path.try_exists()? {
        return Ok(true);
    }

    let shader_last_modified_time =
        std::fs::metadata(shader_file_path)?.modified()?;
    let output_last_modified_time =
        std::fs::metadata(output_path)?.modified()?;

    Ok(shader_last_modified_time > output_last_modified_time)
}

fn compile_shader(shader_file_path: &Path) -> Result<()> {
    let output_path = output_file_for_shader_file(shader_file_path)?;

    if !needs_rebuild(shader_file_path, &output_path).unwrap_or(true) {
        println!(
            "cargo:warning=Skip rebuild for {} because it's up to date",
            shader_file_path.to_str().unwrap()
        );
        return Ok(());
    }

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
        println!(
            "cargo:warning={} -> {}",
            shader_path_str,
            output_path.to_str().unwrap()
        );
        println!("cargo:rerun-if-changed={}", shader_path_str);
    }

    Ok(())
}

fn main() -> Result<()> {
    let all_paths = glob::glob("./**/*.vert")?
        .chain(glob::glob("./**/*.frag")?)
        .chain(glob::glob("./**/*.comp")?);
    for path_entry in all_paths {
        compile_shader(path_entry?.as_path())?;
    }

    Ok(())
}
