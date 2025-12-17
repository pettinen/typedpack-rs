use std::path::Path;

use typedpack_codegen::Type;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("types.rs");

    let file = std::fs::read_to_string("test.tp").unwrap();
    let types = typedpack_codegen::parse(&file).unwrap();

    let mut code = String::new();
    for r#type in types {
        match r#type {
            Type::Struct(r#struct) => {
                code.push_str(&r#struct.rust_struct());
            }
            Type::Enum(r#enum) => {
                code.push_str(&r#enum.rust_enum());
            }
        }
        code.push_str("\n\n");
    }
    std::fs::write(&dest_path, code).unwrap();
    println!("cargo::rerun-if-changed=test.tp");
}
