mod body;
mod header;

use anyhow::{Context, bail};

use super::Raw;
use body::Values;
use header::{Element, Property};

const AXES: [&str; 3] = ["x", "y", "z"];

/// The `vertex` element's `x y z`, and the `face` element's polygons split
/// into triangles fanned from their first corner; other elements and
/// properties are read past.
pub fn parse(bytes: &[u8]) -> anyhow::Result<Raw> {
    let header = header::parse(bytes)?;
    let mut values = Values::new(header.format, &bytes[header.length..])?;
    let mut raw = Raw::default();
    for element in &header.elements {
        if element.count > bytes.len() {
            bail!(
                "{} {} elements cannot fit in the file",
                element.count,
                element.name
            );
        }
        match element.name.as_str() {
            "vertex" => raw.coordinates = vertices(element, &mut values)?,
            "face" => raw.triangles = faces(element, &mut values)?,
            _ => skip(element, &mut values)?,
        }
    }
    if raw.coordinates.is_empty() {
        bail!("no vertex element");
    }
    Ok(raw)
}

fn vertices(element: &Element, values: &mut Values<'_>) -> anyhow::Result<Vec<f64>> {
    let slots: Vec<Option<usize>> = element
        .properties
        .iter()
        .map(|property| match property {
            Property::Scalar { name, .. } => AXES.iter().position(|axis| axis == name),
            Property::List { .. } => None,
        })
        .collect();
    if let Some(missing) = (0..AXES.len()).find(|axis| !slots.contains(&Some(*axis))) {
        bail!("vertex has no {} property", AXES[missing]);
    }
    let mut coordinates = vec![0.0; element.count * AXES.len()];
    for row in coordinates.as_chunks_mut::<3>().0 {
        for (property, slot) in element.properties.iter().zip(&slots) {
            let value = read(property, values)?;
            if let (Some(axis), [single]) = (slot, value.as_slice()) {
                row[*axis] = *single;
            }
        }
    }
    Ok(coordinates)
}

fn faces(element: &Element, values: &mut Values<'_>) -> anyhow::Result<Vec<[u32; 3]>> {
    let mut triangles = Vec::with_capacity(element.count);
    for _ in 0..element.count {
        for property in &element.properties {
            let value = read(property, values)?;
            if let Property::List { name, .. } = property
                && matches!(name.as_str(), "vertex_indices" | "vertex_index")
            {
                let corners = value
                    .iter()
                    .map(|corner| index(*corner))
                    .collect::<Option<Vec<u32>>>()
                    .context("a face index is not a vertex index")?;
                triangles.extend(fan(&corners));
            }
        }
    }
    Ok(triangles)
}

fn skip(element: &Element, values: &mut Values<'_>) -> anyhow::Result<()> {
    for _ in 0..element.count {
        for property in &element.properties {
            read(property, values)?;
        }
    }
    Ok(())
}

fn read(property: &Property, values: &mut Values<'_>) -> anyhow::Result<Vec<f64>> {
    match property {
        Property::Scalar { kind, .. } => Ok(vec![values.next(*kind)?]),
        Property::List { count, item, .. } => {
            let length = index(values.next(*count)?).context("a list length is negative")?;
            (0..length).map(|_| values.next(*item)).collect()
        }
    }
}

fn fan(corners: &[u32]) -> impl Iterator<Item = [u32; 3]> + '_ {
    corners
        .split_first()
        .into_iter()
        .flat_map(|(apex, rest)| rest.windows(2).map(|edge| [*apex, edge[0], edge[1]]))
}

/// A value read from an integer property, as an index; `None` when it is
/// negative, fractional or past `u32`.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is checked to be a whole number within u32 first"
)]
fn index(value: f64) -> Option<u32> {
    (value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value)).then_some(value as u32)
}
