#[cfg(feature = "dot_matrix")]
pub mod dot_matrix;
#[cfg(feature = "rgb_matrix")]
pub mod rgb_matrix;

#[cfg(feature = "dot_matrix")]
pub use dot_matrix::{DotMatrix, COLUMNS, ROWS};

#[cfg(feature = "rgb_matrix")]
pub use rgb_matrix::{COLUMNS, ROWS};
