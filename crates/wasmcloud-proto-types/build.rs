use protobuf_nats_service_generator::NatsServiceGenerator;

fn main() -> std::io::Result<()> {
    // Find all the .proto files
    let protos = std::fs::read_dir("src")?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "proto" {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    prost_build::Config::new()
        .out_dir("src/generated")
        .service_generator(Box::new(NatsServiceGenerator))
        .compile_protos(&protos, &["src/"])?;
    Ok(())
}
