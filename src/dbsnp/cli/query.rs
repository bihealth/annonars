//! Query of dbSNP annotation data.

use std::{io::Write, sync::Arc};

use prost::Message;

use crate::{
    common::{self, cli::extract_chrom, keys, spdi},
    cons::cli::args::vars::ArgsQuery,
    dbsnp,
};

/// Command line arguments for `tsv query` sub command.
#[derive(clap::Parser, Debug, Clone)]
#[command(about = "query dbSNP data stored in RocksDB", long_about = None)]
pub struct Args {
    /// Path to RocksDB directory with data.
    #[arg(long)]
    pub path_rocksdb: String,
    /// Name of the column family to import into.
    #[arg(long, default_value = "dbsnp_data")]
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

/// Meta information as read from database.
#[derive(Debug)]
pub struct Meta {
    /// Genome release of data in database.
    pub genome_release: String,
    /// Name of the database.
    pub db_name: String,
    /// Version of the database.
    pub db_version: String,
}

/// Open RocksDb given path and column family name for data and metadata.
pub fn open_rocksdb<P: AsRef<std::path::Path>>(
    path_rocksdb: P,
    cf_data: &str,
    cf_meta: &str,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    tracing::info!("Opening RocksDB database ...");
    let before_open = std::time::Instant::now();
    let cf_names = &[cf_meta, cf_data];
    let db = Arc::new(rocksdb::DB::open_cf_for_read_only(
        &rocksdb::Options::default(),
        &path_rocksdb,
        cf_names,
        true,
    )?);
    tracing::info!("  reading meta information");
    let meta = {
        let cf_meta = db.cf_handle(cf_meta).unwrap();
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
        Meta {
            genome_release: meta_genome_release,
            db_name: meta_db_name,
            db_version: meta_db_version,
        }
    };

    tracing::info!("  meta:db-name = {}", &meta.db_name);
    tracing::info!("  meta:genome-release = {}", &meta.genome_release);
    tracing::info!("  meta:db-version = {}", &meta.db_version);
    tracing::info!(
        "... opening RocksDB database took {:?}",
        before_open.elapsed()
    );

    Ok((db, meta))
}

/// Open RocksDB database.
pub fn open_rocksdb_from_args(
    args: &Args,
) -> Result<(Arc<rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>>, Meta), anyhow::Error> {
    open_rocksdb(&args.path_rocksdb, &args.cf_name, "meta")
}

fn print_record(
    out_writer: &mut Box<dyn std::io::Write>,
    output_format: common::cli::OutputFormat,
    value: &dbsnp::pbs::Record,
) -> Result<(), anyhow::Error> {
    match output_format {
        common::cli::OutputFormat::Jsonl => {
            writeln!(out_writer, "{}", serde_json::to_string(value)?)?;
        }
    }

    Ok(())
}

/// Query for a single variant in the RocksDB database.
pub fn query_for_variant(
    variant: &common::spdi::Var,
    meta: &Meta,
    db: &rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>,
    cf_data: &Arc<rocksdb::BoundColumnFamily>,
) -> Result<dbsnp::pbs::Record, anyhow::Error> {
    // Split off the genome release (checked) and convert to key as used in database.
    let query = spdi::Var {
        sequence: extract_chrom::from_var(variant, Some(&meta.genome_release))?,
        ..variant.clone()
    };
    // Execute query.
    tracing::debug!("query = {:?}", &query);
    let var: keys::Var = query.into();
    let key: Vec<u8> = var.into();
    let raw_value = db
        .get_cf(cf_data, key)?
        .ok_or_else(|| anyhow::anyhow!("could not find variant in database"))?;
    // Decode via prost.
    dbsnp::pbs::Record::decode(&mut std::io::Cursor::new(&raw_value))
        .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))
}

/// Implementation of `tsv query` sub command.
pub fn run(common: &common::cli::Args, args: &Args) -> Result<(), anyhow::Error> {
    tracing::info!("Starting 'dbsnp query' command");
    tracing::info!("common = {:#?}", &common);
    tracing::info!("args = {:#?}", &args);

    let (db, meta) = open_rocksdb_from_args(args)?;
    let cf_data = db.cf_handle(&args.cf_name).unwrap();

    // Obtain writer to output.
    let mut out_writer = match args.out_file.as_ref() {
        "-" => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
        out_file => {
            let path = std::path::Path::new(out_file);
            Box::new(std::fs::File::create(path).unwrap()) as Box<dyn std::io::Write>
        }
    };

    tracing::info!("Running query...");
    let before_query = std::time::Instant::now();
    if let Some(variant) = args.query.variant.as_ref() {
        print_record(
            &mut out_writer,
            args.out_format,
            &query_for_variant(variant, &meta, &db, &cf_data)?,
        )?;
    } else {
        let (start, stop) = if let Some(position) = args.query.position.as_ref() {
            let position = spdi::Pos {
                sequence: extract_chrom::from_pos(position, Some(&meta.genome_release))?,
                ..position.clone()
            };
            (Some(position.clone()), Some(position))
        } else if let Some(range) = args.query.range.as_ref() {
            let range = spdi::Range {
                sequence: extract_chrom::from_range(range, Some(&meta.genome_release))?,
                ..range.clone()
            };
            let (start, stop) = range.into();
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
            iter.seek(b"")
        }

        // Cast stop to `keys::Pos`.
        let stop = stop.map(|stop| -> keys::Pos { stop.into() });
        if let Some(stop) = stop.as_ref() {
            let stop: Vec<u8> = stop.clone().into();
            tracing::debug!("stop = {:?}", &stop);
        }

        // Iterate over all variants until we are behind stop.
        while iter.valid() {
            if let Some(raw_value) = iter.value() {
                tracing::trace!("iterator at {:?} => {:?}", &iter.key(), &raw_value);
                if let Some(stop) = stop.as_ref() {
                    let iter_key = iter.key().unwrap();
                    let iter_pos: keys::Pos = iter_key.into();

                    if iter_pos.chrom != stop.chrom || iter_pos.pos > stop.pos {
                        break;
                    }
                }

                let record = dbsnp::pbs::Record::decode(&mut std::io::Cursor::new(&raw_value))
                    .map_err(|e| anyhow::anyhow!("failed to decode record: {}", e))?;
                print_record(&mut out_writer, args.out_format, &record)?;
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
            path_rocksdb: String::from("tests/dbsnp/example/dbsnp.brca1.vcf.bgz.db"),
            cf_name: String::from("dbsnp_data"),
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
    fn smoke_query_var_single() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            variant: Some(spdi::Var::from_str("GRCh37:17:41267746:C:CA")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_pos_single() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            position: Some(spdi::Pos::from_str("GRCh37:17:41267752")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }

    #[test]
    fn smoke_query_pos_multi() -> Result<(), anyhow::Error> {
        let (common, args, _temp) = args(ArgsQuery {
            position: Some(spdi::Pos::from_str("GRCh37:17:41267747")?),
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
            range: Some(spdi::Range::from_str("GRCh37:17:40000000:50000000")?),
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
            range: Some(spdi::Range::from_str("GRCh37:17:41267746:41267746")?),
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
            range: Some(spdi::Range::from_str("GRCh37:1:41267747:41267747")?),
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
            range: Some(spdi::Range::from_str("GRCh37:1:1:41267745")?),
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
            range: Some(spdi::Range::from_str("GRCh37:1:41267793:41268000")?),
            ..Default::default()
        });
        run(&common, &args)?;
        let out_data = std::fs::read_to_string(&args.out_file)?;
        insta::assert_snapshot!(&out_data);

        Ok(())
    }
}
