pub mod pcd;
pub mod accumulators;
pub mod proof_aggregator;

#[cfg(test)]
mod tests;

//TODO: Add tracing where missing
//TODO: Add doc where missing
//TODO: Remove dependency from R: RngCore when not needed
//TODO: Do the same with Digest template