
fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OUT_DIR", "./src/proto");

    quix_build::Config::new()
        .service_generator(Box::new(quix_build::Generator))
        .compile_protos(
            &["./proto/net.proto", "./proto/process.proto", "./proto/kv.proto"],
            &["./proto"]).unwrap();


    Ok(())
}