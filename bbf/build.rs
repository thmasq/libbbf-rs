fn main() {
    #[cfg(feature = "uniffi-bindings")]
    uniffi_build::generate_scaffolding("src/bbf.udl").unwrap();
}
