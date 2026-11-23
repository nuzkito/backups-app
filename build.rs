//! Puts the Windows resources (the application icon) in the executable. For other targets,
//! `embed-resource` does nothing.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    embed_resource::compile("assets/app.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
