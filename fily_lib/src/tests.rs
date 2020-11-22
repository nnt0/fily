#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

use crate::check_image_formats::{image_format_guess_from_path, ImageFormatFromPath};
use std::path::PathBuf;

#[test]
fn image_format_from_path_test() {
    assert_eq!(image_format_guess_from_path("image.jpg"), ImageFormatFromPath::Format(image::ImageFormat::Jpeg));
    
    assert_eq!(image_format_guess_from_path("picture.png"), ImageFormatFromPath::Format(image::ImageFormat::Png));

    assert_eq!(image_format_guess_from_path("abcdefg.abc"), ImageFormatFromPath::UnknownFormat(PathBuf::from("abc")));

    assert_eq!(image_format_guess_from_path("maybeNotAnImage."), ImageFormatFromPath::UnknownFormat(PathBuf::from("")));

    assert_eq!(image_format_guess_from_path("notAnImage"), ImageFormatFromPath::NoExtension);
}

use crate::duplicates::crc32_from_bytes;

#[test]
fn crc32_from_bytes_test() {
    let input = b"Super important text it is very important that the crc32 hash of this text never gets calculated";

    assert_eq!(crc32_from_bytes(input), 0x28873A5C);
}
