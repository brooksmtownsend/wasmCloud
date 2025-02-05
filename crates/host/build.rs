fn main() {
    println!("cargo:rerun-if-changed=proto/events.proto");

    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]"); // Optional

    config
        .compile_protos(&["proto/events.proto"], &["proto"])
        .expect("failed to compile proto files");
}
