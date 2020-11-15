use std::fs::{DirEntry};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OUT_DIR", "./src/proto");

    let files: Vec<_> = std::fs::read_dir("./proto")
        .unwrap()
        .map(Result::<DirEntry, _>::unwrap)
        .filter(|f| f.file_type().unwrap().is_file())
        .map(|f| f.path())
        .collect();

    //panic!("{:?}", files);
    quix_build::Config::new()
        .service_generator(quix_build::generator(true))
        .compile_protos(
            &files,
            &[PathBuf::from("./proto")]).unwrap();


    Ok(())
}