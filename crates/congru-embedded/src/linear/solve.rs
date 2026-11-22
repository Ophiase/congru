use crate::Real;

/// Solves `matrix · x = right` for a symmetric positive definite matrix by
/// [Cholesky], in place on the stack; `None` when a pivot is not positive.
///
/// [Cholesky]: https://en.wikipedia.org/wiki/Cholesky_decomposition
#[expect(
    clippy::needless_range_loop,
    reason = "triangular loops read best by their indices"
)]
pub(crate) fn solve_positive<Float: Real, const SIZE: usize>(
    mut matrix: [[Float; SIZE]; SIZE],
    mut right: [Float; SIZE],
) -> Option<[Float; SIZE]> {
    for column in 0..SIZE {
        let mut pivot = matrix[column][column];
        for entry in &matrix[column][..column] {
            pivot -= *entry * *entry;
        }
        if !pivot.strictly_positive() {
            return None;
        }
        let pivot = pivot.root();
        matrix[column][column] = pivot;
        for row in column + 1..SIZE {
            let mut entry = matrix[row][column];
            for earlier in 0..column {
                entry -= matrix[row][earlier] * matrix[column][earlier];
            }
            matrix[row][column] = entry / pivot;
        }
    }
    for row in 0..SIZE {
        for earlier in 0..row {
            right[row] -= matrix[row][earlier] * right[earlier];
        }
        right[row] = right[row] / matrix[row][row];
    }
    for row in (0..SIZE).rev() {
        for later in row + 1..SIZE {
            right[row] -= matrix[later][row] * right[later];
        }
        right[row] = right[row] / matrix[row][row];
    }
    Some(right)
}
