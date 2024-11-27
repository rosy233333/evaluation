#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(noop_waker))]
#![feature(sync_unsafe_cell)]

#[cfg(not(feature = "async-std"))]
#[cfg(test)]
mod test_async_trait;

#[cfg(feature = "async-std")]
#[cfg(test)]
mod test_async_trait_in_async_std;