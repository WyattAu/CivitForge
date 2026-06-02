#![forbid(unsafe_code)]

pub mod types;

#[cfg(feature = "ssr")]
mod client;
#[cfg(feature = "ssr")]
mod issues;
#[cfg(feature = "ssr")]
mod repos;
