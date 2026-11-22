use crate::Real;

const MAX_SWEEPS: usize = 16;

/// The eigenvalues of a symmetric matrix and, by columns, its eigenvectors.
pub(crate) struct Eigen<Float, const SIZE: usize> {
    pub(crate) values: [Float; SIZE],
    pub(crate) vectors: [[Float; SIZE]; SIZE],
}

impl<Float: Real, const SIZE: usize> Eigen<Float, SIZE> {
    pub(crate) fn column(&self, index: usize) -> [Float; SIZE] {
        self.vectors.map(|row| row[index])
    }
}

/// [Cyclic Jacobi]: a few plane rotations with no allocation and small
/// relative errors, which matters in `f32`.
///
/// [Cyclic Jacobi]: https://en.wikipedia.org/wiki/Jacobi_eigenvalue_algorithm
pub(crate) fn symmetric_eigen<Float: Real, const SIZE: usize>(
    mut matrix: [[Float; SIZE]; SIZE],
) -> Eigen<Float, SIZE> {
    let mut vectors = [[Float::ZERO; SIZE]; SIZE];
    for (index, row) in vectors.iter_mut().enumerate() {
        row[index] = Float::ONE;
    }
    for _ in 0..MAX_SWEEPS {
        if off_diagonal(&matrix) <= Float::EPSILON * diagonal(&matrix) {
            break;
        }
        for first in 0..SIZE {
            for second in first + 1..SIZE {
                annihilate(&mut matrix, &mut vectors, first, second);
            }
        }
    }
    let mut values = [Float::ZERO; SIZE];
    for (index, value) in values.iter_mut().enumerate() {
        *value = matrix[index][index];
    }
    Eigen { values, vectors }
}

fn off_diagonal<Float: Real, const SIZE: usize>(matrix: &[[Float; SIZE]; SIZE]) -> Float {
    let mut total = Float::ZERO;
    for (index, row) in matrix.iter().enumerate() {
        for entry in &row[index + 1..] {
            total += entry.magnitude();
        }
    }
    total
}

fn diagonal<Float: Real, const SIZE: usize>(matrix: &[[Float; SIZE]; SIZE]) -> Float {
    let mut total = Float::ZERO;
    for (index, row) in matrix.iter().enumerate() {
        total += row[index].magnitude();
    }
    total
}

#[expect(
    clippy::needless_range_loop,
    reason = "a row and a column updated together by their index"
)]
fn annihilate<Float: Real, const SIZE: usize>(
    matrix: &mut [[Float; SIZE]; SIZE],
    vectors: &mut [[Float; SIZE]; SIZE],
    first: usize,
    second: usize,
) {
    let entry = matrix[first][second];
    if entry == Float::ZERO {
        return;
    }
    let theta = (matrix[second][second] - matrix[first][first]) / (Float::of(2.0) * entry);
    let tangent = Float::ONE / (theta.magnitude() + (theta * theta + Float::ONE).root());
    let tangent = if theta < Float::ZERO {
        -tangent
    } else {
        tangent
    };
    let cosine = Float::ONE / (tangent * tangent + Float::ONE).root();
    let sine = tangent * cosine;
    matrix[first][first] -= tangent * entry;
    matrix[second][second] += tangent * entry;
    matrix[first][second] = Float::ZERO;
    matrix[second][first] = Float::ZERO;
    for other in 0..SIZE {
        if other != first && other != second {
            let (low, high) = (matrix[other][first], matrix[other][second]);
            let (low, high) = (cosine * low - sine * high, sine * low + cosine * high);
            matrix[other][first] = low;
            matrix[first][other] = low;
            matrix[other][second] = high;
            matrix[second][other] = high;
        }
    }
    for row in vectors {
        let (low, high) = (row[first], row[second]);
        row[first] = cosine * low - sine * high;
        row[second] = sine * low + cosine * high;
    }
}
