#[cfg(not(debug_assertions))]
pub mod prod;
#[cfg(not(debug_assertions))]
pub use prod::*;

#[cfg(debug_assertions)]
pub mod dev;
#[cfg(debug_assertions)]
pub use dev::*;
