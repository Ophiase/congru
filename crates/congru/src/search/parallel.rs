/// `operation` over `items`, results in the order of `items`, so that a sum over
/// them does not depend on how many threads ran: on every core with the `parallel`
/// feature, on this thread without it.
#[cfg(feature = "parallel")]
pub fn map<Item: Sync, Output: Send>(
    items: &[Item],
    operation: impl Fn(&Item) -> Output + Sync + Send,
) -> Vec<Output> {
    use rayon::prelude::*;
    items.par_iter().map(operation).collect()
}

/// `operation` over `items`, in order, on this thread: the `parallel` feature is off.
#[cfg(not(feature = "parallel"))]
pub fn map<Item, Output>(items: &[Item], operation: impl Fn(&Item) -> Output) -> Vec<Output> {
    items.iter().map(operation).collect()
}
