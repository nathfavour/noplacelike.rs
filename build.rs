fn main() {
    // Tell Cargo to re-run this build script if the templates directory changes
    println!("cargo:rerun-if-changed=templates");
    
    // Askama will handle template compilation automatically
    // This build script primarily exists to track template changes
}
