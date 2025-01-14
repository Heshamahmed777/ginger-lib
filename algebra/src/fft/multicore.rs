//! This is an interface for dealing with the kinds of
//! parallel computations involved in `snark`. It's
//! currently just a thin wrapper around `rayon`.
use rayon::{self, Scope};

#[derive(Copy, Clone)]
pub(crate) struct Worker {
    cpus: usize,
}

impl Worker {
    pub(crate) fn new() -> Worker {
        let cpus = rayon::current_num_threads();
        Self { cpus }
    }

    pub(crate) fn log_num_cpus(&self) -> u32 {
        crate::log2_floor(self.cpus)
    }

    pub(crate) fn scope<'a, F, R>(&self, elements: usize, f: F) -> R
    where
        F: 'a + Send + FnOnce(&Scope<'a>, usize) -> R,
        R: Send,
    {
        let chunk_size = if elements < self.cpus {
            1
        } else {
            elements / self.cpus
        };

        rayon::scope(move |scope| f(scope, chunk_size))
    }
}
