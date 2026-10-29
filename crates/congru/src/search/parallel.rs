#[cfg(feature = "parallel")]
pub(crate) fn map<Item: Sync, Output: Send>(
    items: &[Item],
    operation: impl Fn(&Item) -> Output + Sync + Send,
) -> Vec<Output> {
    use rayon::prelude::*;
    items.par_iter().map(operation).collect()
}

#[cfg(not(feature = "parallel"))]
pub(crate) fn map<Item, Output>(
    items: &[Item],
    operation: impl Fn(&Item) -> Output,
) -> Vec<Output> {
    items.iter().map(operation).collect()
}
