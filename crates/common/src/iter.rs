/// Returns an iterator that yields the first element of each
/// power of 2 sized chunk.
///
/// Chunks are sized from the highest power of 2 that fits in the remaining
/// length, down to 1, matching the binary representation of the total
/// length.
///
/// **Note**: Chunks larger than `2^max_power` are capped at that size.
pub fn power_chunk_firsts<I>(
    mut iter: I,
    max_power: usize,
) -> impl Iterator<Item = (I::Item, usize)>
where
    I: ExactSizeIterator,
{
    let max_chunk = 1 << max_power;
    core::iter::from_fn(move || {
        let chunk_size = match iter.len() {
            0 => return None,
            n if n >= max_chunk => max_chunk,
            n => 1 << (usize::BITS - n.leading_zeros() - 1),
        };
        let first = iter.next();
        // Consuming chunk-2 because we already consumed the first item.
        (chunk_size > 1).then(|| iter.by_ref().nth(chunk_size - 2));
        first.map(|f| (f, chunk_size))
    })
}
