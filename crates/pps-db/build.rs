//! Recompile le crate quand une migration change : `sqlx::migrate!` embarque
//! les fichiers à la compilation, et une compilation incrémentale ne le
//! verrait pas sinon.

// Les instructions à Cargo passent par la sortie standard.
#![allow(clippy::print_stdout)]

fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
