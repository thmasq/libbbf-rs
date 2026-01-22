fn main() {
    #[cfg(feature = "uniffi-bindings")]
    uniffi::generate_scaffolding("src/bbf.udl").unwrap();
}
