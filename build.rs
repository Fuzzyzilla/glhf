// Generate GL glue.
// From gl_generator README.
extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("gl_bindings.rs")).unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    let mut data = Vec::new();

    // Lol, to ask for GLES3 you say.. GLES2 version 3? weirmd
    Registry::new(Api::Gles2, (3, 2), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut std::io::Cursor::new(&mut data))
        .expect("failed to generate gl bindings");

    let data = String::from_utf8(data).expect("gl bindings are invalid utf8");

    // Make reference to `core` crate instead of `std`
    let data = data.replace("std::mem", "core::mem");
    // std::os::raw is just a public re-export of some of core::ffi, import it as an alias.
    let data = data.replace("use std::os::raw", "use core::ffi as raw");

    file.write_all(data.as_bytes())
        .expect("failed to write gl bindings");
}
