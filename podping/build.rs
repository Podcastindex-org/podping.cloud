fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("plexo-schemas/schema")
        .output_path("plexo-schemas/built")
        .file("plexo-schemas/schema/dev/plexo/plexo_message.capnp")
        .run().expect("schema compiler command");
}