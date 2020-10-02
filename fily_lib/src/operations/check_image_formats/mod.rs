use std::{path::{PathBuf, Path}, fs::canonicalize};
use image::{io::Reader, ImageFormat, ImageError, error::ImageFormatHint};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Checks if the extension of an image matches the actual format
///
/// Returns a `Vec` that contains, in this order, `path_to_file, filename_extension_format, should_be_format`. It could, for example, return
/// `[("./a_file.jpg", "jpg", "png")]` which would mean that the file `a_file.jpg` should actually have `.png` as its extension
/// because that is its actual format.
///
/// This function can work on files that are not actually images without creating an error. Theres no guarantee that it'll report
/// the right thing in this case. For example, it'll report .wav files as a false positive saying that they should have the
/// the .webp extension.
///
/// If it encounters any errors while getting info on files it will just log it
/// (assuming logging is turned on) and ignore the file where the error happened
pub fn check_image_formats<P: AsRef<Path>>(images_to_check: &[P]) -> Vec<(PathBuf, String, String)> {
    let images_to_check = {
        let mut images_to_check_canonicalized = Vec::with_capacity(images_to_check.len());

        for path in images_to_check {
            images_to_check_canonicalized.push(match canonicalize(path) {
                Ok(path) => path,
                Err(e) => {
                    info!("Error accessing {:?} {} skipping this file", path.as_ref().display(), e);
                    continue;
                }
            });
        }

        images_to_check_canonicalized
    };

    trace!("check_image_formats images_to_check: {:?}", images_to_check);

    let mut images_with_wrong_extensions = Vec::new();

    for path in images_to_check {
        let format = {
            let reader = match Reader::open(&path) {
                Ok(reader) => reader,
                Err(e) => {
                    info!("Error instantiating reader of {:?} {} skipping this file", path.display(), e);
                    continue;
                }
            };
            let reader = match reader.with_guessed_format() {
                Ok(reader) => reader,
                Err(e) => {
                    info!("Error guessing format of {:?} {} skipping this file", path.display(), e);
                    continue;
                }
            };

            match reader.format() {
                Some(format) => format.extensions_str()[0],
                None => {
                    info!("Failed to get format of {:?} skipping this file", path.display());
                    continue;
                }
            }
        };
        let format_from_path = match ImageFormat::from_path(&path) {
            Ok(format) => format.extensions_str()[0],
            Err(e) => match e {
                // This is kinda problematic. I've looked through the code of the image crate and
                // found out that if you try to get the image format from a path it should, if it fails, only ever
                // return ImageFormatHint::PathExtension or ImageFormatHint::Unknown but I don't know if
                // this is always going to stay this way so it may be possible that this breaks at some point
                // if I update the image crate. I dislike this piece of code.
                ImageError::Unsupported(unsupported_e) => match unsupported_e.format_hint() {
                    ImageFormatHint::PathExtension(_) => match path.extension() {
                        Some(ext_os) => match ext_os.to_str() {
                            Some(ext) => ext,
                            None => {
                                info!("Failed to convert {:?} to UTF-8 skipping this file", path.extension());
                                continue;
                            }
                        }
                        None => "",
                    },
                    ImageFormatHint::Unknown => "",
                    _ => {
                        warn!("Failed to get current format from path for {:?} ImageError::Unsupported was {} skipping this file", path.display(), unsupported_e.format_hint());
                        continue;
                    }
                }
                _ => {
                    warn!("Failed to get current format from path for {:?} ImageFormat::from_path returned the error {} skipping this file", path.display(), e);
                    continue;
                }
            }
        };

        if format != format_from_path {
            let is_format = format_from_path.to_string();
            let should_be_format = format.to_string();
            images_with_wrong_extensions.push((path, is_format, should_be_format));
        }
    }

    images_with_wrong_extensions
}