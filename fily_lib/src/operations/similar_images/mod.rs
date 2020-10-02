use std::{path::{PathBuf, Path}, fs::canonicalize};
use image::io::Reader;
pub use img_hash::{HashAlg, FilterType};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SimilarImagesOptions {
    pub hash_alg: HashAlg,
    pub filter_type: FilterType,
    pub hash_size: (u32, u32),
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

#[derive(Clone, PartialEq, Eq, Debug)]
struct Image {
    path: PathBuf,
    hash: Option<img_hash::ImageHash>,
}

/// Finds images that are similar to each other
pub fn find_similar_images<P: AsRef<Path>>(images_to_check: &[P], similar_images_options: SimilarImagesOptions) -> Vec<(PathBuf, PathBuf)> {
    let mut images_to_check = {
        let mut images_to_check_canonicalized = Vec::with_capacity(images_to_check.len());

        for path in images_to_check {
            images_to_check_canonicalized.push(Image {
                path: match canonicalize(path) {
                    Ok(path) => path,
                    Err(e) => {
                        info!("Error accessing {:?} {} skipping this file", path.as_ref().display(), e);
                        continue;
                    }
                },
                hash: None,
            });
        }

        images_to_check_canonicalized
    };

    trace!("find_similar_images images_to_check: {:?} similar_images_options: {:?}", images_to_check, similar_images_options);

    let images_to_check_len = images_to_check.len();
    let hasher = img_hash::HasherConfig::new()
        .hash_alg(similar_images_options.hash_alg)
        .resize_filter(similar_images_options.filter_type)
        .hash_size(similar_images_options.hash_size.0, similar_images_options.hash_size.1)
        .to_hasher();

    let mut similar_images = Vec::new();

    for i in 0..images_to_check_len {
        let image1_hash = if images_to_check[i].hash.is_some() {
            images_to_check[i].hash.take().unwrap()
        } else {
            let reader = match Reader::open(&images_to_check[i].path) {
                Ok(reader) => reader,
                Err(e) => {
                    info!("Failed to open {:?} {}", images_to_check[i].path.display(), e);
                    continue;
                }
            };
            let reader = match reader.with_guessed_format() {
                Ok(reader) => reader,
                Err(e) => {
                    info!("Failed to open {:?} {}", images_to_check[i].path.display(), e);
                    continue;
                }
            };
            let image = match reader.decode() {
                Ok(image) => image,
                Err(e) => {
                    info!("Failed to open {:?} {}", images_to_check[i].path.display(), e);
                    continue;
                }
            };
            hasher.hash_image(&image)
        };

        for j in i + 1..images_to_check_len {
            let image2_hash = if let Some(ref hash) = images_to_check[j].hash {
                hash
            } else {
                let reader = match Reader::open(&images_to_check[j].path) {
                    Ok(reader) => reader,
                    Err(e) => {
                        info!("Failed to open {:?} {}", images_to_check[j].path.display(), e);
                        continue;
                    }
                };
                let reader = match reader.with_guessed_format() {
                    Ok(reader) => reader,
                    Err(e) => {
                        info!("Failed to open {:?} {}", images_to_check[j].path.display(), e);
                        continue;
                    }
                };
                let image = match reader.decode() {
                    Ok(image) => image,
                    Err(e) => {
                        info!("Failed to open {:?} {}", images_to_check[j].path.display(), e);
                        continue;
                    }
                };
                let hash = hasher.hash_image(&image);

                images_to_check[j].hash = Some(hash);

                images_to_check[j].hash.as_ref().unwrap()
            };

            let distance = image1_hash.dist(image2_hash);

            if distance <= similar_images_options.threshold {
                similar_images.push((images_to_check[i].path.clone(), images_to_check[j].path.clone()));
            }
        }
    }

    debug!("Found {} similar images", similar_images.len());

    similar_images
}
