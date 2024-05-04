#![no_std]

mod app;
mod display;

pub use app::App;
pub mod interface;

#[cfg(test)]
mod test_infra;
