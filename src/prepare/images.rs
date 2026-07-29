use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

use anyhow::{Context, Result, bail};
use futures::{StreamExt, stream};
use image::{
    ExtendedColorType, ImageReader,
    codecs::jpeg::JpegEncoder,
};
use tempfile::NamedTempFile;

use super::{
    cards::CardDatabase,
    download::Downloader,
    limits::YearLimits,
};

const IMAGE_URL: &str = "https://images.ygoprodeck.com/images/cards_cropped";
const MAX_CONCURRENT_DOWNLOADS: usize = 8;
const JPEG_QUALITY: u8 = 90;

#[derive(Clone, Copy)]
enum Preparation {
    Downloaded,
    Converted,
    Reused,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ImageKind {
    Jpeg,
    Png,
}

pub(super) async fn fetch(
    downloader: &Downloader,
    workspace: &Path,
    cards: &CardDatabase,
    limit_lists: &[YearLimits],
) -> Result<()> {
    let image_ids = required_image_ids(
        limit_lists.iter().flat_map(YearLimits::ids),
        cards,
    )?;
    let images_directory = workspace.join("assets/ot/images");
    let results = stream::iter(image_ids.iter().copied())
        .map(|image_id| {
            let destination = images_directory.join(format!("{image_id}.jpg"));
            async move {
                match image_kind(&destination) {
                    Some(ImageKind::Jpeg) => return Ok(Preparation::Reused),
                    Some(ImageKind::Png) => {
                        convert_png_to_jpeg(&destination)?;
                        return Ok(Preparation::Converted);
                    }
                    None => {}
                }

                let url = format!("{IMAGE_URL}/{image_id}.jpg");
                downloader
                    .download_checked(&url, &destination, validate_image)
                    .await
                    .with_context(|| format!("failed to download center image {image_id}"))?;
                if image_kind(&destination) == Some(ImageKind::Png) {
                    convert_png_to_jpeg(&destination)?;
                }
                Ok(Preparation::Downloaded)
            }
        })
        .buffer_unordered(MAX_CONCURRENT_DOWNLOADS)
        .collect::<Vec<Result<Preparation>>>()
        .await;

    let mut downloaded = 0;
    let mut converted = 0;
    let mut failures = Vec::new();
    for result in results {
        match result {
            Ok(Preparation::Downloaded) => downloaded += 1,
            Ok(Preparation::Converted) => converted += 1,
            Ok(Preparation::Reused) => {}
            Err(error) => failures.push(format!("{error:#}")),
        }
    }

    if !failures.is_empty() {
        let details = failures
            .iter()
            .take(10)
            .map(|failure| format!("- {failure}"))
            .collect::<Vec<_>>()
            .join("\n");
        bail!(
            "{} center image download(s) failed:\n{details}",
            failures.len()
        );
    }

    println!(
        "prepared {} center images ({downloaded} downloaded, {converted} converted, {} reused)",
        image_ids.len(),
        image_ids.len() - downloaded - converted
    );
    Ok(())
}

fn required_image_ids(
    card_ids: impl IntoIterator<Item = u32>,
    cards: &CardDatabase,
) -> Result<BTreeSet<u32>> {
    card_ids
        .into_iter()
        .map(|id| cards.image_id(id))
        .collect()
}

fn validate_image(path: &Path) -> Result<()> {
    if image_kind(path).is_some() {
        Ok(())
    } else {
        bail!("download is neither JPEG nor PNG")
    }
}

fn image_kind(path: &Path) -> Option<ImageKind> {
    const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

    let mut header = [0; 8];
    if File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_err()
    {
        return None;
    }

    if header[..2] == [0xff, 0xd8] {
        Some(ImageKind::Jpeg)
    } else if header == PNG_HEADER {
        Some(ImageKind::Png)
    } else {
        None
    }
}

fn convert_png_to_jpeg(path: &Path) -> Result<()> {
    let image = ImageReader::open(path)
        .with_context(|| format!("failed to open PNG {}", path.display()))?
        .with_guessed_format()
        .context("failed to detect downloaded image format")?
        .decode()
        .with_context(|| format!("failed to decode PNG {}", path.display()))?
        .to_rgb8();
    let parent = path
        .parent()
        .context("center image destination has no parent")?;
    let mut temporary =
        NamedTempFile::new_in(parent).context("failed to create converted image file")?;

    {
        let mut output = BufWriter::new(temporary.as_file_mut());
        JpegEncoder::new_with_quality(&mut output, JPEG_QUALITY)
            .encode(
                image.as_raw(),
                image.width(),
                image.height(),
                ExtendedColorType::Rgb8,
            )
            .context("failed to encode center image as JPEG")?;
        output.flush().context("failed to flush converted JPEG")?;
    }

    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("failed to replace {} with converted JPEG", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use image::ImageReader;
    use tempfile::tempdir;

    use super::{ImageKind, convert_png_to_jpeg, image_kind, required_image_ids};
    use crate::prepare::cards::CardDatabase;

    #[test]
    fn recognizes_jpeg_and_png_headers() {
        let temp = tempdir().unwrap();
        let jpeg = temp.path().join("valid.jpg");
        let png = temp.path().join("png-with-jpg-extension.jpg");
        let invalid = temp.path().join("invalid.jpg");
        fs::write(&jpeg, [0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0]).unwrap();
        fs::write(&png, [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).unwrap();
        fs::write(&invalid, b"not jpeg").unwrap();

        assert!(matches!(image_kind(&jpeg), Some(ImageKind::Jpeg)));
        assert!(matches!(image_kind(&png), Some(ImageKind::Png)));
        assert!(image_kind(&invalid).is_none());
        assert!(image_kind(&temp.path().join("missing.jpg")).is_none());
    }

    #[test]
    fn converts_png_content_at_jpg_path_to_jpeg() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("center.jpg");
        image::save_buffer_with_format(
            &path,
            &[255, 0, 0],
            1,
            1,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .unwrap();

        convert_png_to_jpeg(&path).unwrap();

        assert!(matches!(image_kind(&path), Some(ImageKind::Jpeg)));
        let decoded = ImageReader::open(&path)
            .unwrap()
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert_eq!((decoded.width(), decoded.height()), (1, 1));
    }

    #[test]
    fn maps_card_ids_to_unique_center_images() {
        let temp = tempdir().unwrap();
        let cards_path = temp.path().join("ot.json");
        fs::write(
            &cards_path,
            r#"[
                {"id":1,"image":101,"type":["魔法","通常"]},
                {"id":2,"image":101,"type":["魔法","通常"]},
                {"id":3,"image":103,"type":["陷阱","通常"]}
            ]"#,
        )
        .unwrap();
        let cards = CardDatabase::load(&cards_path).unwrap();

        let images = required_image_ids([3, 1, 2, 1], &cards).unwrap();

        assert_eq!(images.into_iter().collect::<Vec<_>>(), [101, 103]);
    }
}
