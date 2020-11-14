use pb_rs::types::FileDescriptor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OUT_DIR", "./src/proto");
    let cfg = pb_rs::ConfigBuilder::new(
        &["./proto/net.proto", "./proto/process.proto"],
        None,
        Some(&"./src/proto"),
        &["./proto"],
    )
        .unwrap()
        .owned(true)
        .single_module(true)
        .build();

    for c in &cfg {
        FileDescriptor::write_proto(c).unwrap()
    }

    Ok(())
}