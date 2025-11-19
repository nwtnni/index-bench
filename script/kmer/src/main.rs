// https://github.com/nicolasgarza/fetchkmer/blob/d910161007cb7e4c3396a49998081db6d6a1f134/src/compress.rs

use ascii::AsciiChar;
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use std::fs::File;
use std::io::Write;
use std::{fs, io, path::Path};

fn main() {}

const NEWLINE: u8 = b'\n';

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BaseType {
    A = 0b00,
    T = 0b01,
    C = 0b10,
    G = 0b11,
}

impl BaseType {
    #[inline]
    pub fn from_ascii(byte: u8) -> Option<Self> {
        match AsciiChar::from_ascii(byte).ok()? {
            AsciiChar::A | AsciiChar::a => Some(BaseType::A),
            AsciiChar::T | AsciiChar::t => Some(BaseType::T),
            AsciiChar::C | AsciiChar::c => Some(BaseType::C),
            AsciiChar::G | AsciiChar::g => Some(BaseType::G),
            _ => None,
        }
    }
}

/// Strip headers/newlines, keep only A/T/C/G as BaseType.
pub fn extract_bases(input: &[u8]) -> Vec<BaseType> {
    let mut out = Vec::with_capacity(input.len());
    let mut in_header = false;

    for &b in input {
        if b == NEWLINE {
            in_header = false;
            continue;
        }
        if in_header {
            continue;
        }
        if b == b'>' {
            in_header = true;
            continue;
        }
        if let Some(bt) = BaseType::from_ascii(b) {
            out.push(bt);
        }
    }
    out
}

/// Pack bases into 2-bit representation; returns (packed_bytes, num_bases).
pub fn compress_bases_2bit(bases: &[BaseType]) -> io::Result<(Vec<u8>, usize)> {
    let mut buffer: Vec<u8> = Vec::with_capacity((bases.len() + 3) / 4);
    {
        let mut writer = BitWriter::endian(&mut buffer, BigEndian);
        for &b in bases {
            writer.write::<2, u32>(b as u32)?;
        }
        writer.byte_align()?;
    }
    Ok((buffer, bases.len()))
}

/// Read a FASTA, compress to 2-bit, write:
/// [8-byte little-endian base count][packed 2-bit bases...]
pub fn compress_fasta_file_single(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let file = fs::read(input_path)?;
    let bases = extract_bases(&file);
    let (compressed, count) = compress_bases_2bit(&bases)?;

    let mut f = File::create(output_path)?;
    f.write_all(&(count as u32).to_le_bytes())?;
    f.write_all(&compressed)?;
    Ok(())
}
