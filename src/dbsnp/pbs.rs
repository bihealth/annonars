//! Data structures for (de-)serialization as generated by `prost-build`.

use noodles_vcf::variant::record::AlternateBases;

pub use crate::pbs::dbsnp::Record;
use noodles_vcf::variant::record_buf::info::field;

impl Record {
    /// Creates a new `Record` from a VCF record and allele number.
    pub fn from_vcf_allele(
        record: &noodles_vcf::variant::RecordBuf,
        allele_no: usize,
    ) -> Result<Self, anyhow::Error> {
        let chrom = record.reference_sequence_name().to_string();
        let pos: usize = record
            .variant_start()
            .expect("Telomeric breakends not supported")
            .get();
        let pos: i32 = i32::try_from(pos)?;
        let ref_allele = record.reference_bases().to_string();
        let alt_allele = record
            .alternate_bases()
            .iter()
            .nth(allele_no)
            .ok_or_else(|| anyhow::anyhow!("no such allele: {}", allele_no))??
            .to_string();
        let rs_id = if let Some(Some(field::Value::Integer(rs))) = record.info().get("RS") {
            *rs
        } else {
            anyhow::bail!("no rs id in dbSNP record")
        };

        Ok(Record {
            chrom,
            pos,
            ref_allele,
            alt_allele,
            rs_id,
        })
    }
}
