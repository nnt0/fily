use std::{path::{Path, PathBuf}, io, fmt, error::Error};
use image::{io::Reader, ImageFormat, ImageError, error::ImageFormatHint};
use crate::fily_err::{Context, FilyError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Debug)]
pub enum CheckImageFormatsError {
    /// If something went wrong while guessing the path from the content
    ContentGuessError(FilyError<io::Error>),

    /// If the paths extension is not a known extension
    UnknownPathExtension,

    /// If the path has no extension
    NoPathExtension,
}

impl Error for CheckImageFormatsError {}

impl fmt::Display for CheckImageFormatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Checks if the extension of an image matches the actual format
///
/// Returns a tuple of two `Vec`s. The first one contain the paths for which no error occured. 
/// The second one contains the paths for which an error occured.
///
/// The first `Vec` contains, in this order, `path_to_file, filename_extension_format, should_be_format`. It could, for example, return
/// `[("./a_file.jpg", "jpg", "png")]` which would mean that the file `a_file.jpg` should actually have `.png` as its extension
/// because that is its actual format.
///
/// This function can work on files that are not actually images without creating an error. There is no guarantee that it'll report
/// the right thing in this case. For example, it'll report .wav files as a false positive saying that they should have the
/// the .webp extension.
pub fn check_image_formats<P: AsRef<Path>>(images_to_check: &[P]) -> (Vec<(&Path, String, String)>, Vec<(&Path, CheckImageFormatsError)>) {
    let images_to_check: Vec<&Path> = images_to_check.iter().map(AsRef::as_ref).collect();

    trace!("check_image_formats images_to_check: {:?}", images_to_check);

    let mut images_with_wrong_extensions = Vec::new();
    let mut errors = Vec::new();

    for path in images_to_check {
        let format = match image_format_guess_from_content(&path) {
            Ok(format) => format.extensions_str()[0],
            Err(e) => {
                errors.push((path, CheckImageFormatsError::ContentGuessError(e)));
                continue;
            }
        };

        let format_from_path = match image_format_guess_from_path(&path) {
            ImageFormatFromPath::Format(format) => format.extensions_str()[0],
            ImageFormatFromPath::UnknownFormat(_) => {
                errors.push((path, CheckImageFormatsError::UnknownPathExtension));
                continue;
            }
            ImageFormatFromPath::NoExtension => {
                errors.push((path, CheckImageFormatsError::NoPathExtension));
                continue;
            }
        };

        if format != format_from_path {
            let is_format = format_from_path.to_string();
            let should_be_format = format.to_string();

            images_with_wrong_extensions.push((path, is_format, should_be_format));
        }
    }

    (images_with_wrong_extensions, errors)
}

/// Guesses the extension of an image from its contents
///
/// Note that this function doesn't check if the file is actually an image
/// which can lead to false guesses. For example a .wav file will get detectet as an .webp.
/// Try to make sure to only pass actual images to this function or otherwise it can't be
/// guaranteed that the guess will be anywhere near correct.
///
/// # Errors
///
/// This function returns an error if
///
/// * the path points to a folder
/// * the path doesn't exist/file can't be opened or read from
/// * it was unable to determine the format of the image
pub fn image_format_guess_from_content(path: impl AsRef<Path>) -> Result<ImageFormat, FilyError<io::Error>> {
    let path = path.as_ref();

    trace!("image_extension_guess_from_content path: {:?}", path.display());

    let reader = Reader::open(&path)
        .with_context(|| format!("Error instantiating reader of {:?}", path.display()))?
        .with_guessed_format()
        .with_context(|| format!("Error guessing format of {:?}", path.display()))?;

    reader.format()
        .ok_or_else(|| FilyError::new_with_context(io::Error::new(io::ErrorKind::Other, "Unknown"), || format!("Failed to get format of {:?}", path.display())))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ImageFormatFromPath {
    /// Contains the format of the image guessed from its name
    Format(image::ImageFormat),

    /// If the extension is unknown it will return the extension without the .
    UnknownFormat(PathBuf),

    /// If the name of the file has no extension
    NoExtension,
}

/// Gets the extension of an image for an image from its name
pub fn image_format_guess_from_path(path: impl AsRef<Path>) -> ImageFormatFromPath {
    let path = path.as_ref();

    trace!("image_extension_guess_from_path path: {:?}", path.display());

    match ImageFormat::from_path(&path) {
        Ok(format) => ImageFormatFromPath::Format(format),
        Err(e) => match e {
            // This is kinda problematic. I've looked through the code of the image crate and
            // found out that if you try to get the image format from a path it should, if it fails, only ever
            // return ImageFormatHint::PathExtension or ImageFormatHint::Unknown but I don't know if
            // this is always going to stay this way so it may be possible that this breaks at some point
            // if I update the image crate. I dislike this piece of code.
            // TODO: Write a test for this
            ImageError::Unsupported(unsupported_e) => match unsupported_e.format_hint() {
                ImageFormatHint::PathExtension(ext) => ImageFormatFromPath::UnknownFormat(ext),
                ImageFormatHint::Unknown => ImageFormatFromPath::NoExtension,
                _ => unreachable!("ImageFormatHint wasn't PathExtension nor Unknown")
            }
            _ => unreachable!("ImageError wasn't the Unsupported variant")
        }
    }
}
