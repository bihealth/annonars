//! Autosomal counts.

use byteorder::{ByteOrder, LittleEndian};
use noodles::vcf::variant::record::AlternateBases;

use crate::common::noodles;

/// Record type for storing AN, AC_hom, AC_het counts for autosomal chromosomes.
#[derive(Default, Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Counts {
    /// Total number of alleles.
    pub an: u32,
    /// Number of hom. alt. alleles.
    pub ac_hom: u32,
    /// Number of het. alt. alleles.
    pub ac_het: u32,
}

impl Counts {
    /// Create from the given VCF record.
    pub fn from_vcf_allele(value: &noodles::vcf::variant::RecordBuf, _allele_no: usize) -> Self {
        tracing::trace!("@ {:?}", &value);
        assert_eq!(
            value.alternate_bases().len(),
            1,
            "only one alternate allele is supported",
        );

        let ac = common::noodles::get_i32(value, "AC").expect("could not find: INFO/AC") as u32;
        let ac_hom = common::noodles::get_i32(value, "nhomalt")
            .expect("could not find: INFO/nhomalt") as u32;
        let an = common::noodles::get_i32(value, "AN").expect("could not find: INFO/AN") as u32;

        Counts {
            ac_hom,
            ac_het: ac - 2 * ac_hom,
            an,
        }
    }

    /// Read from buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            an: LittleEndian::read_u32(&buf[0..4]),
            ac_hom: LittleEndian::read_u32(&buf[4..8]),
            ac_het: LittleEndian::read_u32(&buf[8..12]),
        }
    }

    /// Write to buffer.
    pub fn to_buf(&self, buf: &mut [u8]) {
        LittleEndian::write_u32(&mut buf[0..4], self.an);
        LittleEndian::write_u32(&mut buf[4..8], self.ac_hom);
        LittleEndian::write_u32(&mut buf[8..12], self.ac_het);
    }
}

/// Record type for the "autosomal" column family.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// Counts from gnomAD exomes.
    pub gnomad_exomes: Counts,
    /// Counts from gnomAD genomes.
    pub gnomad_genomes: Counts,
}

impl Record {
    /// Read from buffer.
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            gnomad_exomes: Counts::from_buf(&buf[0..16]),
            gnomad_genomes: Counts::from_buf(&buf[16..32]),
        }
    }

    /// Write to buffer.
    pub fn to_buf(&self, buf: &mut [u8]) {
        self.gnomad_exomes.to_buf(&mut buf[0..16]);
        self.gnomad_genomes.to_buf(&mut buf[16..32]);
    }

    /// Return number of byes in buffer.
    pub fn buf_len() -> usize {
        32
    }
}
