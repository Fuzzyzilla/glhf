// Generate GL glue.
// From gl_generator README.
extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("gl_bindings.rs")).unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    // Lol, to ask for GLES3 you say.. GLES2 version 3? weirmd
    Registry::new(Api::Gles2, (3, 2), Profile::Core, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
