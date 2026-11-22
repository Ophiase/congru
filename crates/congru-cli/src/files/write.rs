use std::fmt::Write as _;

use super::Raw;

#[derive(Debug, Clone, Copy)]
enum Encoding {
    Ascii,
    BinaryFloat,
    BinaryDouble,
}

fn xyz(raw: &Raw) -> Vec<u8> {
    let mut text = String::from("# x y z\n");
    for row in raw.coordinates.as_chunks::<3>().0 {
        let _ = writeln!(text, "{} {} {} 0.5", row[0], row[1], row[2]);
    }
    text.into_bytes()
}

fn ply(raw: &Raw, encoding: Encoding) -> Vec<u8> {
    let (format, scalar) = match encoding {
        Encoding::Ascii => ("ascii", "double"),
        Encoding::BinaryFloat => ("binary_little_endian", "float"),
        Encoding::BinaryDouble => ("binary_little_endian", "double"),
    };
    let mut bytes = format!(
        "ply\nformat {format} 1.0\ncomment written by a test\n\
         element vertex {}\nproperty {scalar} x\nproperty {scalar} y\n\
         property uchar red\nproperty {scalar} z\n\
         element face {}\nproperty list uchar int vertex_indices\nend_header\n",
        raw.coordinates.len() / 3,
        raw.triangles.len(),
    )
    .into_bytes();
    for row in raw.coordinates.as_chunks::<3>().0 {
        push_scalar(&mut bytes, encoding, row[0]);
        push_scalar(&mut bytes, encoding, row[1]);
        push_byte(&mut bytes, encoding, u8::MAX);
        push_scalar(&mut bytes, encoding, row[2]);
    }
    for triangle in &raw.triangles {
        push_byte(&mut bytes, encoding, 3);
        for corner in triangle {
            push_index(&mut bytes, encoding, *corner);
        }
    }
    bytes
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the tests write coordinates exact in f32"
)]
fn push_scalar(bytes: &mut Vec<u8>, encoding: Encoding, value: f64) {
    match encoding {
        Encoding::Ascii => bytes.extend(format!("{value} ").as_bytes()),
        Encoding::BinaryFloat => bytes.extend((value as f32).to_le_bytes()),
        Encoding::BinaryDouble => bytes.extend(value.to_le_bytes()),
    }
}

fn push_byte(bytes: &mut Vec<u8>, encoding: Encoding, value: u8) {
    match encoding {
        Encoding::Ascii => bytes.extend(format!("{value} ").as_bytes()),
        Encoding::BinaryFloat | Encoding::BinaryDouble => bytes.push(value),
    }
}

fn push_index(bytes: &mut Vec<u8>, encoding: Encoding, corner: u32) {
    match encoding {
        Encoding::Ascii => bytes.extend(format!("{corner}\n").as_bytes()),
        Encoding::BinaryFloat | Encoding::BinaryDouble => {
            bytes.extend(i32::try_from(corner).unwrap_or(i32::MAX).to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetrahedron() -> Raw {
        Raw {
            coordinates: vec![
                0.0, 0.0, 0.0, 1.5, 0.0, -0.25, 0.0, 2.0, 0.5, -0.75, 0.125, 3.0,
            ],
            triangles: vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]],
        }
    }

    #[test]
    fn xyz_reads_back_what_was_written() {
        let raw = tetrahedron();
        let read = super::super::xyz::parse(&xyz(&raw)).unwrap();
        assert_eq!(
            read,
            Raw {
                coordinates: raw.coordinates,
                triangles: Vec::new(),
            }
        );
    }

    #[test]
    fn ply_reads_back_what_was_written_in_every_encoding() {
        let raw = tetrahedron();
        for encoding in [
            Encoding::Ascii,
            Encoding::BinaryFloat,
            Encoding::BinaryDouble,
        ] {
            let read = super::super::ply::parse(&ply(&raw, encoding)).unwrap();
            assert_eq!(read, raw, "{encoding:?}");
        }
    }

    #[test]
    fn a_quad_is_fanned_into_two_triangles() {
        let quad = "ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\n\
                    property float y\nproperty float z\nelement face 1\n\
                    property list uchar int vertex_indices\nend_header\n\
                    0 0 0\n1 0 0\n1 1 0\n0 1 0\n4 0 1 2 3\n";
        let read = super::super::ply::parse(quad.as_bytes()).unwrap();
        assert_eq!(read.triangles, vec![[0, 1, 2], [0, 2, 3]]);
    }

    #[test]
    fn a_truncated_binary_body_is_an_error() {
        let bytes = ply(&tetrahedron(), Encoding::BinaryDouble);
        assert!(super::super::ply::parse(&bytes[..bytes.len() - 3]).is_err());
    }
}
