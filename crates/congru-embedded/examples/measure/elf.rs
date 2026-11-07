use anyhow::{Context, bail};

const SECTION_ALLOCATED: u32 = 2;
const SECTION_WRITABLE: u32 = 1;
const SYMBOL_TABLE: u32 = 2;
const SYMBOL_BYTES: usize = 16;
const SECTION_HEADER_OFFSET: usize = 0x20;
const SECTION_ENTRY_SIZE: usize = 0x2e;
const SECTION_COUNT: usize = 0x30;

/// What a 32-bit little-endian image costs a board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Allocated and read-only: code and constants, in flash.
    pub flash_bytes: usize,
    /// Software helpers for `f64`: none means the image never computes in it.
    pub double_helpers: usize,
}

struct Section {
    kind: u32,
    flags: u32,
    offset: usize,
    size: usize,
    link: usize,
}

impl Image {
    /// # Errors
    /// Anything but a well-formed ELF32 with its symbols.
    pub fn read(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.get(..5) != Some(b"\x7fELF\x01") {
            bail!("not a 32-bit ELF image");
        }
        let start = word(bytes, SECTION_HEADER_OFFSET)?;
        let entry = half(bytes, SECTION_ENTRY_SIZE)?;
        let sections = (0..half(bytes, SECTION_COUNT)?)
            .map(|index| section(bytes, start + index * entry))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let flash_bytes = sections
            .iter()
            .filter(|found| {
                found.flags & SECTION_ALLOCATED != 0 && found.flags & SECTION_WRITABLE == 0
            })
            .map(|found| found.size)
            .sum();
        let symbols = sections
            .iter()
            .find(|found| found.kind == SYMBOL_TABLE)
            .context("the image has no symbols")?;
        let names = sections
            .get(symbols.link)
            .context("the symbols have no names")?;
        Ok(Self {
            flash_bytes,
            double_helpers: double_helpers(bytes, symbols, names)?,
        })
    }
}

fn double_helpers(bytes: &[u8], symbols: &Section, names: &Section) -> anyhow::Result<usize> {
    let table = bytes
        .get(symbols.offset..symbols.offset + symbols.size)
        .context("a symbol table past the end of the image")?;
    let text = bytes
        .get(names.offset..names.offset + names.size)
        .context("symbol names past the end of the image")?;
    let mut count = 0;
    for symbol in table.chunks_exact(SYMBOL_BYTES) {
        let name = text.get(word(symbol, 0)?..).unwrap_or_default();
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        let name = &name[..end];
        if name.starts_with(b"__aeabi_d") || name.starts_with(b"__aeabi_f2d") {
            count += 1;
        }
    }
    Ok(count)
}

fn section(bytes: &[u8], at: usize) -> anyhow::Result<Section> {
    Ok(Section {
        kind: u32_at(bytes, at + 4)?,
        flags: u32_at(bytes, at + 8)?,
        offset: word(bytes, at + 16)?,
        size: word(bytes, at + 20)?,
        link: word(bytes, at + 24)?,
    })
}

fn u32_at(bytes: &[u8], at: usize) -> anyhow::Result<u32> {
    let raw = bytes
        .get(at..at + 4)
        .context("past the end of the image")?
        .try_into()?;
    Ok(u32::from_le_bytes(raw))
}

fn word(bytes: &[u8], at: usize) -> anyhow::Result<usize> {
    Ok(usize::try_from(u32_at(bytes, at)?)?)
}

fn half(bytes: &[u8], at: usize) -> anyhow::Result<usize> {
    let raw = bytes
        .get(at..at + 2)
        .context("past the end of the image")?
        .try_into()?;
    Ok(usize::from(u16::from_le_bytes(raw)))
}
