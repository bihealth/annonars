// The custom build script, needed as we use flatbuffers.

fn main() {
    println!("cargo:rerun-if-changed=annonars/cons/v1/base.proto");
    println!("cargo:rerun-if-changed=annonars/dbsnp/v1/base.proto");
    println!("cargo:rerun-if-changed=annonars/gnomad/v1/mtdna.proto");
    println!("cargo:rerun-if-changed=annonars/gnomad/v1/nuclear.proto");
    println!("cargo:rerun-if-changed=annonars/gnomad/v1/vep_common.proto");
    println!("cargo:rerun-if-changed=annonars/gnomad/v1/vep_gnomad2.proto");
    println!("cargo:rerun-if-changed=annonars/gnomad/v1/vep_gnomad3.proto");
    println!("cargo:rerun-if-changed=annonars/helixmtdb/v1/base.proto");
    prost_build::Config::new()
        .protoc_arg("-Isrc/proto")
        // Add serde serialization and deserialization to the generated
        // code.
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        // Rename the field attributes such that we can properly decode
        // ucsc-annotation TSV file with serde.
        .field_attribute(
            "annonars.cons.v1.base.Record.chrom",
            "#[serde(rename = \"chromosome\")]",
        )
        .field_attribute(
            "annonars.cons.v1.base.Record.begin",
            "#[serde(rename = \"start\")]",
        )
        .field_attribute(
            "annonars.cons.v1.base..Record.end",
            "#[serde(rename = \"stop\")]",
        )
        // Define the protobuf files to compile.
        .compile_protos(
            &[
                "annonars/cons/v1/base.proto",
                "annonars/dbsnp/v1/base.proto",
                "annonars/gnomad/v1/mtdna.proto",
                "annonars/gnomad/v1/nuclear.proto",
                "annonars/gnomad/v1/vep_common.proto",
                "annonars/gnomad/v1/vep_gnomad2.proto",
                "annonars/gnomad/v1/vep_gnomad3.proto",
                "annonars/helixmtdb/v1/base.proto",
            ],
            &["src/"],
        )
        .unwrap();
}