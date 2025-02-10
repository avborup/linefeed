use crate::vm::{
    runtime_value::{number::RuntimeNumber, range::RuntimeRange},
    RuntimeError,
};

/// Resolve a user-provided index into the actual index. E.g. -1 should resolve to the last
/// element.
pub fn resolve_index(len: usize, index: &RuntimeNumber) -> Result<usize, RuntimeError> {
    let n = index.floor_int();

    let i = if n.is_negative() {
        len as isize - n.abs()
    } else {
        n
    };

    if i < 0 || i as usize >= len {
        return Err(RuntimeError::IndexOutOfBounds(n, len));
    }

    Ok(i as usize)
}

/// Resolve the actual start and end indices of a slice operation, much like `resolve_index`.
pub fn resolve_slice_indices(
    len: usize,
    range: &RuntimeRange,
) -> Result<(usize, usize), RuntimeError> {
    let start = range.start.unwrap_or(0).into();
    let end = (range.end.unwrap_or(len as isize) - 1).into();

    let start_i = resolve_index(len, &start)?;
    let end_i = resolve_index(len, &end)?;

    Ok((start_i, end_i))
}
