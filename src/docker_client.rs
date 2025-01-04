use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::path::PathBuf;
use tar::Archive;

const AUTH_HOST: &str = "https://auth.docker.io/token";
const AUTH_SERVICE: &str = "registry.docker.io";
const REGISTRY_HOST: &str = "registry.hub.docker.com";

#[derive(Deserialize)]
struct AuthToken {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ImageManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: i64,
    #[serde(rename = "mediaType")]
    media_type: String,
    #[serde(default)]
    config: Config,
    #[serde(default)]
    layers: Vec<Layer>,
}

#[derive(Debug, Deserialize, Default)]
struct Config {
    #[serde(rename = "mediaType")]
    media_type: String,
    size: i64,
    digest: String,
}

#[derive(Debug, Deserialize)]
struct Layer {
    #[serde(rename = "mediaType")]
    media_type: String,
    size: i64,
    digest: String,
    #[serde(default)]
    urls: Vec<String>,
}

pub struct DockerClient {
    client: reqwest::blocking::Client,
}

impl DockerClient {
    pub fn new() -> DockerClient {
        return DockerClient {
            client: reqwest::blocking::Client::new(),
        };
    }

    pub fn download_image(&self, full_image: &str, path: &PathBuf) -> Result<()> {
        let (image, tag) = match full_image.split_once(":") {
            Some((image, tag)) => (image, tag),
            None => (full_image, "latest"),
        };
        let token = self.get_token(image)?;
        let manifest = self.get_manifest(image, tag, &token.token)?;
        println!("Downloading image: {:#?}", manifest);

        for l in &manifest.layers {
            self.pull_layer_and_unpack(image, &l.digest, &token.token, path)?;
        }

        return Ok(());
    }

    fn get_token(&self, image: &str) -> Result<AuthToken> {
        return Ok(self
            .client
            .get(AUTH_HOST)
            .query(&[
                ("service", AUTH_SERVICE),
                ("scope", &format!("repository:library/{}:pull", image)),
            ])
            .send()
            .context("sending auth request")?
            .json::<AuthToken>()
            .context("converting token from json")?);
    }

    fn get_manifest(&self, image: &str, tag: &str, token_str: &String) -> Result<ImageManifest> {
        // TODO: check different image manifest types
        return Ok(self
            .client
            .get(format!(
                "https://{}/v2/library/{}/manifests/{}",
                REGISTRY_HOST, image, tag
            ))
            .header("Authorization", format!("Bearer {}", token_str))
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .context("sending manifest request")?
            .json::<ImageManifest>()
            .context("converting manifest from json")?);
    }

    fn pull_layer_and_unpack(
        &self,
        image: &str,
        digest: &String,
        token_str: &String,
        path: &PathBuf,
    ) -> Result<()> {
        let layer_bytes = self
            .client
            .get(format!(
                "https://{}/v2/library/{}/blobs/{}",
                REGISTRY_HOST, image, digest
            ))
            .header("Authorization", format!("Bearer {}", token_str))
            .send()
            .context("sending layer blob request")?
            .bytes()
            .context("getting layer bytes")?;
        let reader = std::io::Cursor::new(layer_bytes);
        let tar = GzDecoder::new(reader);
        let mut archive: Archive<_> = Archive::new(tar);

        archive
            .unpack(path)
            .context("attempting to decompress image layer")?;
        return Ok(());
    }
}
