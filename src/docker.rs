use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::path::PathBuf;
use tar::Archive;

const AUTH_HOST: &str = "https://auth.docker.io/token";
const AUTH_SERVICE: &str = "registry.docker.io";
const REGISTRY_HOST: &str = "registry.hub.docker.com";

#[derive(Deserialize)]
pub(crate) struct AuthToken {
    token: String,
}

#[derive(Deserialize)]
pub(crate) struct ImageManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: i64,
    #[serde(rename = "mediaType")]
    media_type: String,
    #[serde(default)]
    config: Config,
    layers: Vec<Layer>,
}

#[derive(Deserialize, Default)]
pub(crate) struct Config {
    #[serde(rename = "mediaType")]
    media_type: String,
    size: i64,
    digest: String,
}

#[derive(Deserialize)]
pub(crate) struct Layer {
    #[serde(rename = "mediaType")]
    media_type: String,
    size: i64,
    digest: String,
    #[serde(default)]
    urls: Vec<String>,
}

fn parse_image(image: &String) -> Result<(&str, &str)> {
    let image_pieces: Vec<&str> = image.split(":").collect();
    if image_pieces.len() != 2 {
        return Err(anyhow!("error: incorrectly formatted image"));
    }
    return Ok((image_pieces[0], image_pieces[1]));
}
pub fn download_image(image: &String, path: &PathBuf) -> Result<()> {
    let (im_name, im_tag) = parse_image(image)?;
    let client = reqwest::blocking::Client::new();

    let token = client
        .get(AUTH_HOST)
        .query(&[
            ("service", AUTH_SERVICE),
            ("scope", &format!("repository:library/{}:pull", im_name)),
        ])
        .send()
        .context("sending auth request")?
        .json::<AuthToken>()
        .context("converting token from json")?;

    let manifest: ImageManifest = client
        .get(format!(
            "https://{}/v2/library/{}/manifests/{}",
            REGISTRY_HOST, im_name, im_tag
        ))
        .header("Authorization", format!("Bearer {}", token.token))
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()
        .context("sending manifest request")?
        .json::<ImageManifest>()
        .context("converting manifest from json")?;

    for l in &manifest.layers {
        let layer = client
            .get(format!(
                "https://{}/v2/library/{}/blobs/{}",
                REGISTRY_HOST, im_name, l.digest
            ))
            .header("Authorization", format!("Bearer {}", token.token))
            .send()
            .context("sending layer blob request")?
            .bytes()
            .context("getting layer bytes")?;

        let reader = std::io::Cursor::new(layer);
        let tar = GzDecoder::new(reader);
        let mut archive: Archive<_> = Archive::new(tar);

        archive
            .unpack(path)
            .context("attempting to decompress image layer")?;
    }

    Ok(())
}
