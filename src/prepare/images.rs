use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::Path,
};

use anyhow::{Context, Result, bail};
use futures::{StreamExt, stream};

use super::{
    cards::CardDatabase,
    download::Downloader,
    limits::YearLimits,
};

const IMAGE_URL: &str = "https://images.ygoprodeck.com/images/cards_cropped";
const MAX_CONCURRENT_DOWNLOADS: usize = 8;

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
                if is_supported_image(&destination) {
                    return Ok(false);
                }

                let url = format!("{IMAGE_URL}/{image_id}.jpg");
                downloader
                    .download_checked(&url, &destination, validate_image)
                    .await
                    .with_context(|| format!("failed to download center image {image_id}"))?;
                Ok(true)
            }
        })
        .buffer_unordered(MAX_CONCURRENT_DOWNLOADS)
        .collect::<Vec<Result<bool>>>()
        .await;

    let mut downloaded = 0;
    let mut failures = Vec::new();
    for result in results {
        match result {
            Ok(true) => downloaded += 1,
            Ok(false) => {}
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
        "prepared {} center images ({downloaded} downloaded, {} reused)",
        image_ids.len(),
        image_ids.len() - downloaded
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
    if is_supported_image(path) {
        Ok(())
    } else {
        bail!("download is neither JPEG nor PNG")
    }
}

fn is_supported_image(path: &Path) -> bool {
    const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

    let mut header = [0; 8];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_ok()
        && (header[..2] == [0xff, 0xd8] || header == PNG_HEADER)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{is_supported_image, required_image_ids};
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

        assert!(is_supported_image(&jpeg));
        assert!(is_supported_image(&png));
        assert!(!is_supported_image(&invalid));
        assert!(!is_supported_image(&temp.path().join("missing.jpg")));
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
