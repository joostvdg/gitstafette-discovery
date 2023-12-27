use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    //intellij does not easily find types when not adding code to src
    // tonic_build::compile_protos("./protos/gitstafette_discovery.proto")?;
    // tonic_build::compile_protos("./protos/gitstafette_info.proto")?;

    // https://timvw.be/2022/04/28/notes-on-using-grpc-with-rust-and-tonic/
    let original_out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let out_dir = "./src/bin";

    tonic_build::configure()
        .out_dir(out_dir)
        .file_descriptor_set_path(original_out_dir.join("gitstafette_discovery.bin"))
        .compile(&["./protos/gitstafette_discovery.proto"], &["proto"])?;

    tonic_build::configure()
        .out_dir(out_dir)
        .file_descriptor_set_path(original_out_dir.join("gitstafette_info.bin"))
        .compile(&[ "./protos/gitstafette_info.proto"], &["proto"])?;


    Ok(())
}
