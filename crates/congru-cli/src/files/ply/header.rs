use anyhow::{Context, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Ascii,
    BinaryLittleEndian,
}

/// The numeric types a PLY property may hold, under both of their names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
}

impl Scalar {
    fn named(name: &str) -> anyhow::Result<Self> {
        Ok(match name {
            "char" | "int8" => Self::Int8,
            "uchar" | "uint8" => Self::Uint8,
            "short" | "int16" => Self::Int16,
            "ushort" | "uint16" => Self::Uint16,
            "int" | "int32" => Self::Int32,
            "uint" | "uint32" => Self::Uint32,
            "float" | "float32" => Self::Float32,
            "double" | "float64" => Self::Float64,
            _ => bail!("unknown property type {name}"),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Property {
    Scalar {
        name: String,
        kind: Scalar,
    },
    List {
        name: String,
        count: Scalar,
        item: Scalar,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub count: usize,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub format: Format,
    pub elements: Vec<Element>,
    /// Bytes from the start of the file to the first byte of the body.
    pub length: usize,
}

const END: &[u8] = b"end_header";

pub fn parse(bytes: &[u8]) -> anyhow::Result<Header> {
    let end = bytes
        .windows(END.len())
        .position(|window| window == END)
        .context("no end_header line")?;
    let length = bytes[end..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|newline| end + newline + 1)
        .context("no line break after end_header")?;
    let (format, elements) = declarations(&bytes[..end])?;
    Ok(Header {
        format,
        elements,
        length,
    })
}

fn declarations(header: &[u8]) -> anyhow::Result<(Format, Vec<Element>)> {
    let text = std::str::from_utf8(header).context("header is not text")?;
    let mut lines = text.lines().map(str::trim);
    if lines.next() != Some("ply") {
        bail!("not a PLY file");
    }
    let mut format = None;
    let mut elements: Vec<Element> = Vec::new();
    for line in lines {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["format", "ascii", _] => format = Some(Format::Ascii),
            ["format", "binary_little_endian", _] => format = Some(Format::BinaryLittleEndian),
            ["format", other, ..] => bail!("format {other} is not supported"),
            ["element", name, count] => elements.push(Element {
                name: (*name).to_owned(),
                count: count.parse().context("element count")?,
                properties: Vec::new(),
            }),
            ["property", ..] => elements
                .last_mut()
                .context("property before any element")?
                .properties
                .push(property(&words[1..])?),
            [] | ["comment" | "obj_info", ..] => {}
            _ => bail!("unexpected header line: {line}"),
        }
    }
    Ok((format.context("no format line")?, elements))
}

fn property(words: &[&str]) -> anyhow::Result<Property> {
    Ok(match words {
        ["list", count, item, name] => Property::List {
            name: (*name).to_owned(),
            count: Scalar::named(count)?,
            item: Scalar::named(item)?,
        },
        [kind, name] => Property::Scalar {
            name: (*name).to_owned(),
            kind: Scalar::named(kind)?,
        },
        _ => bail!("malformed property: {}", words.join(" ")),
    })
}
