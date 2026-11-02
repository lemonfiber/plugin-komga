//! A library for comics and one for books, each added where its folder exists and no
//! library reads it, and none Komga already holds renamed, re-pointed or removed.

use std::path::Path;

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::media::BOOKS;
use lemonfiber_ports::service::{Failure, FiledAt, Furnished, Furnishing};
use serde::Deserialize;

use super::{Komga, LIBRARIES, LIBRARIES_UNREAD, REFUSED};

const FILESYSTEM: &str = "/api/v1/filesystem";

/// The media type comics are filed under, in the stack manifest's vocabulary.
const COMICS: &str = "comics";

const REFUSED_THE_FOLDER: std::ops::Range<u16> = 400..500;

/// One media type Komga reads as a library, and the name its library is given.
struct Kind {
    media_type: &'static str,
    name: &'static str,
}

const TAKEN: [Kind; 2] = [
    Kind {
        media_type: COMICS,
        name: "Comics",
    },
    Kind {
        media_type: BOOKS,
        name: "Books",
    },
];

/// A library Komga holds, by its name and the folder it reads.
#[derive(Deserialize)]
struct Shelf {
    name: String,
    root: String,
}

#[derive(Deserialize)]
struct Listed {
    directories: Vec<Directory>,
}

#[derive(Deserialize)]
struct Directory {
    path: String,
}

/// Whether `status` is Komga refusing the folder asked about, rather than the
/// administrator.
fn refused_the_folder(status: u16) -> bool {
    REFUSED_THE_FOLDER.contains(&status) && status != REFUSED
}

/// `path` with no trailing `/`.
fn bare(path: &str) -> &str {
    path.trim_end_matches('/')
}

impl Komga {
    /// Whether `folder` exists, as Komga lists the folder holding it, a folder Komga
    /// refuses to list answered as not.
    async fn exists(&self, folder: &str) -> Result<bool, Failure> {
        let Some(parent) = Path::new(bare(folder)).parent().and_then(Path::to_str) else {
            return Ok(false);
        };
        let asked = serde_json::json!({ "path": parent, "showFiles": false }).to_string();
        let response = self.as_admin(Method::Post, FILESYSTEM, Some(asked)).await?;
        if refused_the_folder(response.status) {
            return Ok(false);
        }
        let listed: Listed = self
            .endpoint
            .decode(&response, "the folders Komga lists could not be read")?;
        Ok(listed
            .directories
            .iter()
            .any(|directory| bare(&directory.path) == bare(folder)))
    }

    /// Add a library of `kind` reading `folder`: whether Komga now reads it, Komga refusing
    /// it answered as not.
    async fn added(&self, kind: &Kind, folder: &str) -> Result<bool, Failure> {
        let asked = serde_json::json!({ "name": kind.name, "root": folder }).to_string();
        let response = self.as_admin(Method::Post, LIBRARIES, Some(asked)).await?;
        if refused_the_folder(response.status) {
            return Ok(false);
        }
        self.endpoint.expect_success(&response).map(|()| true)
    }

    /// Whether Komga now reads `folder` as a library of `kind`, adding one where the folder
    /// exists and neither a library reads it nor one is already named for `kind`.
    async fn reads_as(
        &self,
        kind: &Kind,
        folder: &str,
        libraries: &mut Vec<Shelf>,
    ) -> Result<bool, Failure> {
        if libraries
            .iter()
            .any(|library| bare(&library.root) == bare(folder))
        {
            return Ok(true);
        }
        if libraries
            .iter()
            .any(|library| library.name.eq_ignore_ascii_case(kind.name))
        {
            return Ok(false);
        }
        if !self.exists(folder).await? || !self.added(kind, folder).await? {
            return Ok(false);
        }
        libraries.push(Shelf {
            name: kind.name.to_owned(),
            root: folder.to_owned(),
        });
        Ok(true)
    }
}

#[async_trait]
impl Furnishing for Komga {
    async fn furnish(&self, filed: &[FiledAt]) -> Result<Vec<Furnished>, Failure> {
        let mut libraries: Vec<Shelf> = self.read(LIBRARIES, LIBRARIES_UNREAD).await?;
        let mut furnished = Vec::with_capacity(filed.len());
        for one in filed {
            let held = match TAKEN.iter().find(|kind| kind.media_type == one.media_type) {
                None => false,
                Some(kind) => self.reads_as(kind, &one.folder, &mut libraries).await?,
            };
            furnished.push(Furnished {
                media_type: one.media_type.clone(),
                held,
            });
        }
        Ok(furnished)
    }
}
