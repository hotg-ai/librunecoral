use std::path::PathBuf;

use bindgen::Builder;

fn main() {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let header_file = manifest_dir.join("runecoral.h");

    let bindings = Builder::default()
        .header(header_file.display().to_string())
        .dynamic_library_name("RuneCoral")
        .dynamic_link_require_all(true)
        .derive_debug(true)
        .derive_copy(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .unwrap();

    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    let dest = out_dir.join("bindings.rs");

    bindings.write_to_file(dest).unwrap();
}
