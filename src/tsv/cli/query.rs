//! Query for variants, positions, or ranges.

use std::sync::Arc;

use crate::{
    common::{self, keys, spdi},
    tsv::{coding, schema},
};

///

/// Command line arguments for `tsv query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "import tsv data into rocksdb", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "tsv_data")]
    pub cf_name: String,
    /// Output file (default is stdout == "-").
    #[arg(long, default_value = "-")]
    pub out_file: String,
    /// Output format.
    #[arg(long, default_value = "jsonl")]
    pub out_format: common::cli::OutputFormat,

    /// Variant or position to query for.
    #[command(flatten)]
    pub query: ArgsQuery,
}

/// Argument group for specifying one of variant, position, or range.
#[derive(clap::Args, Debug, Clone, Default)]
#[group(required = true, multiple = false)]
pub struct ArgsQuery {
    /// Specify variant to query for.
    #[arg(long, group = "query")]
    pub variant: Option<spdi::Var>,
    /// Specify position to query for.
    #[arg(long, group = "query")]
    pub position: Option<spdi::Pos>,
    /// Specify range to query for.
    #[arg(long, group = "query")]
    pub range: Option<spdi::Range>,
    /// Query for all variants.
    #[arg(long, group = "query")]
    pub all: bool,
}

/// Meta information as read from database.
#[derive(Debug)]
struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
    /// Name of the database.
    pub db_name: String,
    /// Version of the database.
    pub db_version: String,
    /// Schema of the database.
    pub db_schema: schema::FileSchema,
    /// Inference configuration.
    pub db_infer_config: schema::infer::Config,
}

/// Open RocksDB database.
fn open_rocksdb(
    args: &Args,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &["meta", &args.cf_name];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        &args.path_rocksdb,
        cf_names,
        true,
    )?);
    tracing::info!("  reading meta information");
    let meta = {
        let cf_meta = db.cf_handle("meta").unwrap();
        let meta_db_name = String::from_utf8(
            db.get_cf(&cf_meta, "db-name")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:db-schema"))?,
        )?;
        let meta_genome_release = String::from_utf8(
            db.get_cf(&cf_meta, "genome-release")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:genome-release"))?,
        )?;
        let meta_db_version = String::from_utf8(
            db.get_cf(&cf_meta, "db-version")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:db-schema"))?,
        )?;
        let meta_db_schema = String::from_utf8(
            db.get_cf(&cf_meta, "db-schema")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:db-schema"))?,
        )?;
        let meta_db_infer_config = String::from_utf8(
            db.get_cf(&cf_meta, "db-infer-config")?
                .ok_or_else(|| anyhow::anyhow!("missing value meta:db-infer-config"))?,
        )?;
        Meta {
            genome_release: meta_genome_release,
            db_name: meta_db_name,
            db_version: meta_db_version,
            db_schema: serde_json::from_str(&meta_db_schema)?,
            db_infer_config: serde_json::from_str(&meta_db_infer_config)?,
        }
    };

    tracing::info!("  meta:db-name = {}", &meta.db_name);
    tracing::info!("  meta:genome-release = {}", &meta.genome_release);
    tracing::info!("  meta:db-version = {}", &meta.db_version);
    tracing::info!(
        "  meta:db-schema = {}",
        &serde_json::to_string(&meta.db_schema)?
    );
    tracing::info!(
        "  meta:db-infer-config = {}",
        &serde_json::to_string(&meta.db_infer_config)?
    );
    tracing::info!(
        "... opening RocksDB database took {:?}",
        before_open.elapsed()
    );

    Ok((db, meta))
}

/// Get chromosome from the SPDI variant.
///
/// If the optional genome release was given then it is compared to the one specified
/// in `meta` and stripped (comparision is case insensitive).
fn extract_chrom_var(variant: &spdi::Var, meta: &Meta) -> Result<String, anyhow::Error> {
    if variant.sequence.contains(":") {
        let mut iter = variant.sequence.rsplitn(2, ":");
        let chromosome = iter.next().unwrap();
        if let Some(genome_release) = iter.next() {
            if genome_release.to_lowercase() != meta.genome_release.to_lowercase() {
                return Err(anyhow::anyhow!(
                    "genome release mismatch (lowercase): expected {}, got {}",
                    meta.genome_release,
                    genome_release
                ));
            }
        }
        Ok(chromosome.to_owned())
    } else {
        Ok(variant.sequence.clone())
    }
}

/// Get chromosome from the SPDI position.
///
/// See `extract_chrom_var` for details.
fn extract_chrom_pos(pos: &spdi::Pos, meta: &Meta) -> Result<String, anyhow::Error> {
    if pos.sequence.contains(":") {
        let mut iter = pos.sequence.rsplitn(2, ":");
        let chromosome = iter.next().unwrap();
        if let Some(genome_release) = iter.next() {
            if genome_release.to_lowercase() != meta.genome_release.to_lowercase() {
                return Err(anyhow::anyhow!(
                    "genome release mismatch (lowercase): expected {}, got {}",
                    meta.genome_release,
                    genome_release
                ));
            }
        }
        Ok(chromosome.to_owned())
    } else {
        Ok(pos.sequence.clone())
    }
}

/// Get chromosome from the SPDI range.
///
/// See `extract_chrom_var` for details.
fn extract_chrom_range(range: &spdi::Range, meta: &Meta) -> Result<String, anyhow::Error> {
    if range.sequence.contains(":") {
        let mut iter = range.sequence.rsplitn(2, ":");
        let chromosome = iter.next().unwrap();
        if let Some(genome_release) = iter.next() {
            if genome_release.to_lowercase() != meta.genome_release.to_lowercase() {
                return Err(anyhow::anyhow!(
                    "genome release mismatch (lowercase): expected {}, got {}",
                    meta.genome_release,
                    genome_release
                ));
            }
        }
        Ok(chromosome.to_owned())
    } else {
        Ok(range.sequence.clone())
    }
}

/// Print values to stdout.
fn print_values(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    meta: &Meta,
    values: Vec<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    match output_format {
        common::cli::OutputFormat::Jsonl => {
            let mut map = serde_json::Map::new();
            for (col, value) in meta.db_schema.columns.iter().zip(values.iter()) {
                if !value.is_null() {
                    map.insert(col.name.clone(), value.clone());
                }
            }
            writeln!(
                out_writer,
                "{}",
                serde_json::to_string(&serde_json::Value::Object(map))?
            )?;
        }
    }

    Ok(())
}

/// Perform query for variant.
fn query_for_variant(
    variant: &spdi::Var,
    meta: &Meta,
    db: &Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>,
    cf_data: Arc<rocksdb::BoundColumnFamily>,
    ctx: coding::Context,
) -> Result<Vec<serde_json::Value>, anyhow::Error> {
    // Split off the genome release (checked) and convert to key as used in database.
    let query = spdi::Var {
        sequence: extract_chrom_var(variant, meta)?,
        ..variant.clone()
    };
    tracing::debug!("query = {:?}", &query);
    let var: keys::Var = query.into();
    let key: Vec<u8> = var.into();
    let raw_value = db
        .get_cf(&cf_data, &key)?
        .ok_or_else(|| anyhow::anyhow!("could not find variant in database"))?;
    let values = ctx.decode_values(&raw_value)?;

    Ok(values)
}

/// Implementation of `tsv query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'tsv query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    let (db, meta) = open_rocksdb(args)?;
    let cf_data = db.cf_handle(&args.cf_name).unwrap();
    let ctx = coding::Context::new(meta.db_infer_config.clone(), meta.db_schema.clone());

    // Obtain writer to output.
    let mut out_writer = match args.out_file.as_ref() {
        "-" => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
        out_file => {
            let path = std::path::Path::new(out_file);
            Box::new(std::fs::File::create(&path).unwrap()) as Box<dyn std::io::Write>
        }
    };

    tracing::info!("Running query...");
    let before_query = std::time::Instant::now();
    if let Some(variant) = args.query.variant.as_ref() {
        print_values(
            &mut out_writer,
            args.out_format,
            &meta,
            query_for_variant(variant, &meta, &db, cf_data, ctx)?,
        )?;
    } else {
        let (start, stop) = if let Some(position) = args.query.position.as_ref() {
            let position = spdi::Pos {
                sequence: extract_chrom_pos(position, &meta)?,
                ..position.clone()
            };
            (Some(position.clone()), Some(position.clone()))
        } else if let Some(range) = args.query.range.as_ref() {
            let range = spdi::Range {
                sequence: extract_chrom_range(range, &meta)?,
                ..range.clone()
            };
            let (start, stop) = range.clone().into();
            (Some(start), Some(stop))
        } else if args.query.all {
            (None, None)
        } else {
            unreachable!()
        };

        tracing::debug!("start = {:?}, stop = {:?}", &start, &stop);

        // Obtain iterator and seek to start.
        let mut iter = db.raw_iterator_cf(&cf_data);
        if let Some(start) = start {
            let pos: keys::Pos = start.into();
            let key: Vec<u8> = pos.into();
            tracing::debug!("seeking to key {:?}", &key);
            iter.seek(&key);
        } else {
            iter.seek(&b"")
        }

        // Cast stop to `keys::Pos`.
        let stop = stop.map(|stop| -> keys::Pos { stop.into() });
        if let Some(stop) = stop.as_ref() {
            let stop: Vec<u8> = stop.clone().into();
            tracing::debug!("stop = {:?}", &stop);
        }

        // Iterate over all variants until we are behind stop.
        while iter.valid() {
            if let Some(value) = iter.value() {
                tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &value);
                if let Some(stop) = stop.as_ref() {
                    let iter_key = iter.key().unwrap();
                    let iter_pos: keys::Pos = iter_key.into();

                    if &iter_pos > stop {
                        break;
                    }
                }

                let values = ctx.decode_values(&value)?;
                print_values(&mut out_writer, args.out_format, &meta, values)?;
                iter.next();
            } else {
                break;
            }
        }
    }
    tracing::info!("... done querying in {:?}", before_query.elapsed());

    tracing::info!("All done. Have a nice day!");
    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    use temp_testdir::TempDir;

    fn args(query: ArgsQuery) -> (common::cli::Args, Args, TempDir) {
        let temp = TempDir::default();
        let common = common::cli::Args {
            verbose: clap_verbosity_flag::Verbosity::new(1, 0),
        };
        let args = Args {
            path_rocksdb: String::from("tests/tsv/example/data.tsv.gz.db"),
            cf_name: String::from("tsv_data"),
            out_file: temp.join("out").to_string_lossy().to_string(),
            out_format: common::cli::OutputFormat::Jsonl,
            query,
        };

        (common, args, temp)
    }

    #[test]
    fn smoke_query_all() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            all: true,
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_var() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            variant: Some(spdi::Var::from_str("GRCh37:1:1000:A:T")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_pos() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            position: Some(spdi::Pos::from_str("GRCh37:1:1000")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_find_all() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:1:1000:1001")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_find_first() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:1:1000:1000")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_find_second() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:1:1001:1001")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_find_none_smaller() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:1:1:999")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_range_find_none_larger() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            range: Some(spdi::Range::from_str("GRCh37:1:1002:2000")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
