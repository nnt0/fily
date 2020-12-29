use std::{path::Path, io, fmt, error::Error};
use image::io::Reader;
pub use img_hash::{HashAlg, FilterType};
use crate::fily_err::{Context, FilyError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Used as options for `find_similar_images`
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SimilarImagesOptions {
    /// What hashing algorithm to use
    pub hash_alg: HashAlg,

    /// Which filter it should use
    pub filter_type: FilterType,

    /// Hash width and height
    pub hash_size: (u32, u32),

    /// How close the images have to be to be considered similar
    pub threshold: u32,
}

impl Default for SimilarImagesOptions {
    fn default() -> Self {
        SimilarImagesOptions {
            hash_alg: HashAlg::Gradient,
            filter_type: FilterType::Lanczos3,
            hash_size: (8, 8),
            threshold: 31,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Image<'a> {
    path: &'a Path,
    hash: Option<img_hash::ImageHash>,
}

#[derive(Debug)]
pub enum HashImageError {
    IOError(FilyError<io::Error>),
    ImageError(FilyError<image::ImageError>)
}

impl Error for HashImageError {}

impl fmt::Display for HashImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<FilyError<io::Error>> for HashImageError {
    fn from(err: FilyError<io::Error>) -> Self {
        HashImageError::IOError(err)
    }
}

impl From<FilyError<image::ImageError>> for HashImageError {
    fn from(err: FilyError<image::ImageError>) -> Self {
        HashImageError::ImageError(err)
    }
}

fn hash_image(path: &Path, hasher: &img_hash::Hasher) -> Result<img_hash::ImageHash, HashImageError> {
    let reader = Reader::open(path)
        .with_context(|| format!("Failed to open {:?}", path.display()))?
        .with_guessed_format()
        .with_context(|| format!("Failed to guess image format {:?}", path.display()))?;

    let image = reader.decode()
        .with_context(|| format!("Failed to decode {:?}", path.display()))?;

    let hash = hasher.hash_image(&image);

    Ok(hash)
}

/// Finds images that are similar to each other
///
/// You can specify on how exactly it should find the pictures with `SimilarImagesOptions`
///
/// I recommend reading the docs of the crate `img_hash` for an explanation on what the hash
/// algorithms und filters do
///
/// If you're lazy you can just use `SimilarImagesOptions::default()` for a configuration
/// that works decently well
pub fn find_similar_images<P: AsRef<Path>>(images_to_check: &[P], similar_images_options: SimilarImagesOptions) -> (Vec<(&Path, &Path)>, Vec<(&Path, HashImageError)>) {
    let images_to_check: Vec<&Path> = images_to_check.iter().map(AsRef::as_ref).collect();

    trace!("find_similar_images images_to_check: {:?} similar_images_options: {:?}", images_to_check, similar_images_options);

    let mut images_to_check: Vec<Image<'_>> = images_to_check.into_iter().map(|path| {
            Image {
                path,
                hash: None,
            }
        }).collect();

    let images_to_check_len = images_to_check.len();
    let hasher = img_hash::HasherConfig::new()
        .hash_alg(similar_images_options.hash_alg)
        .resize_filter(similar_images_options.filter_type)
        .hash_size(similar_images_options.hash_size.0, similar_images_options.hash_size.1)
        .to_hasher();

    let mut similar_images = Vec::new();
    let mut errors = Vec::new();

    for i in 0..images_to_check_len {
        let image1_hash = if images_to_check[i].hash.is_some() {
            images_to_check[i].hash.take().unwrap()
        } else {
            match hash_image(images_to_check[i].path, &hasher) {
                Ok(hash) => hash,
                Err(e) => {
                    errors.push((images_to_check[i].path, e));
                    continue;
                }
            }
        };

        for j in i + 1..images_to_check_len {
            let image2_hash = if let Some(ref hash) = images_to_check[j].hash {
                hash
            } else {
                let hash = match hash_image(images_to_check[j].path, &hasher) {
                    Ok(hash) => hash,
                    Err(e) => {
                        errors.push((images_to_check[j].path, e));
                        continue;
                    }
                };

                images_to_check[j].hash = Some(hash);

                images_to_check[j].hash.as_ref().unwrap()
            };

            let distance = image1_hash.dist(image2_hash);

            if distance <= similar_images_options.threshold {
                similar_images.push((images_to_check[i].path, images_to_check[j].path));
            }
        }
    }

    debug!("Found {} similar images", similar_images.len());

    (similar_images, errors)
}
