fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("podping-schemas/schema")
        .output_path("podping-schemas/built")
        .file("podping-schemas/schema/org/podcastindex/podping/hivewriter/podping_hive_transaction.capnp")
        .file("podping-schemas/schema/org/podcastindex/podping/hivewriter/podping_hive_write.capnp")
        .file("podping-schemas/schema/org/podcastindex/podping/hivewriter/podping_medium.capnp")
        .file("podping-schemas/schema/org/podcastindex/podping/hivewriter/podping_reason.capnp")
        .file("podping-schemas/schema/org/podcastindex/podping/hivewriter/podping_write.capnp")
        .file("podping-schemas/schema/org/podcastindex/podping/hivewriter/podping_write_error.capnp")
        .run().expect("schema compiler command");
}