use std::{
    env,
    path::{Path, PathBuf},
};

fn get_output_path() -> PathBuf {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    PathBuf::from(path)
}

fn main() {
    let env_gmsh = "GMSH_SDK_DIR";
    let env = env::var(env_gmsh);
    match env {
        Ok(sdk_dir) => {
            println!("cargo:rerun-if-env-changed={}", env_gmsh);

            let sdk_dir = Path::new(&sdk_dir);
            let lib_dir = sdk_dir.join("lib");
            let header_dir = sdk_dir.join("include").join("gmshc.h");

            let dylib_name = format!("{}gmsh{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
            if lib_dir.join(dylib_name).exists() || lib_dir.join("gmsh.lib").exists() {
                println!("cargo:rustc-link-search={}", lib_dir.display());
                println!("cargo:rustc-link-lib=gmsh");

                let bindings = bindgen::Builder::default()
                    .header(header_dir.display().to_string())
                    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
                    .generate()
                    .expect("Unable to generate bindings");

                bindings
                    .write_to_file("src/bindings.rs")
                    .expect("Failed to write bindings");

                let lib_files = lib_dir.read_dir().unwrap();
                for file in lib_files {
                    let file_path = file.unwrap().path();
                    let flag = file_path
                        .extension()
                        .unwrap()
                        .eq_ignore_ascii_case(env::consts::DLL_EXTENSION);
                    if flag {
                        let out_path = get_output_path();
                        let gmsh_out_file = out_path.join(file_path.file_name().unwrap());
                        std::fs::copy(file_path, gmsh_out_file).unwrap();
                    }
                }
            } else {
                panic!(
                    "{} is set to {:?}, but no shared library files were found there",
                    env_gmsh, lib_dir
                );
            }
        }
        Err(_) => {
            panic!("Please define GMSH_SDK_DIR in the environment variable");
        }
    }
}
