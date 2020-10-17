#[cfg(feature = "duplicates")]
pub mod duplicates;

#[cfg(feature = "rename")]
pub mod rename;

#[cfg(feature = "find")]
pub mod find;

#[cfg(feature = "move_files")]
pub mod move_files;

#[cfg(feature = "similar_images")]
pub mod similar_images;

#[cfg(feature = "check_image_formats")]
pub mod check_image_formats;