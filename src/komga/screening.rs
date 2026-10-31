//! What Komga holds, read as its administrator reads it: each series as an item, and a
//! series or one of its books as a title. A member of the household is somebody
//! `identity.source` names, and Komga speaks for none of them, so everything asked for one
//! member is unsupported, and so is what is playing: Komga reads, it plays nothing.

use async_trait::async_trait;
use lemonfiber_adapter_kit::endpoint::form_encoded;
use lemonfiber_adapter_kit::picture;
use lemonfiber_ports::service::{
    Failure, Holds, HowFar, Image, Item, ItemDetail, ItemProgress, Medium, Picture, Playback,
    Screening, PICTURE_MOST,
};
use serde::Deserialize;

use super::{an_id, Komga, ABSENT, BOOKS, SERIES};

const NO_MEMBERS: &str =
    "Komga speaks for none of the household's accounts, so nothing is asked for one";

const NO_DEVICES: &str = "Komga opens and ends no device's session for lemonfiber";

const NOTHING_PLAYS: &str = "Komga is read, and plays nothing";

const BY_TITLE: &str = "metadata.titleSort,asc";

const DATE_LENGTH: usize = 10;

#[derive(Deserialize)]
struct Page<T> {
    content: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Series {
    id: String,
    name: String,
    #[serde(default)]
    books_count: u32,
    #[serde(default)]
    metadata: Described,
    #[serde(default)]
    books_metadata: Described,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Book {
    id: String,
    name: String,
    #[serde(default)]
    metadata: Described,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Described {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    release_date: Option<String>,
}

impl Described {
    fn released(&self) -> Option<String> {
        self.release_date.clone().filter(|date| a_date(date))
    }

    fn year(&self) -> Option<u16> {
        self.released()
            .and_then(|date| date.get(..4).and_then(|year| year.parse().ok()))
    }

    fn overview(&self) -> Option<String> {
        self.summary.clone().filter(|said| !said.trim().is_empty())
    }
}

/// Whether `said` is a calendar date written as `YYYY-MM-DD`.
pub(super) fn a_date(said: &str) -> bool {
    said.len() == DATE_LENGTH
        && said.bytes().enumerate().all(|(at, mark)| match at {
            4 | 7 => mark == b'-',
            _ => mark.is_ascii_digit(),
        })
}

fn titled(named: Option<&String>, name: &str) -> String {
    named
        .filter(|title| !title.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| name.to_owned())
}

impl Series {
    fn item(&self) -> Item {
        Item {
            id: self.id.clone(),
            title: titled(self.metadata.title.as_ref(), &self.name),
            year: self.books_metadata.year(),
            medium: Medium::Other,
            holds: Holds {
                poster: self.books_count > 0,
                backdrop: false,
                plays: false,
            },
        }
    }

    fn detail(self) -> ItemDetail {
        let item = self.item();
        ItemDetail {
            item,
            overview: self
                .metadata
                .overview()
                .or_else(|| self.books_metadata.overview()),
            minutes: None,
            genres: self.metadata.genres,
            certificate: None,
            released: self.books_metadata.released(),
            seasons: Vec::new(),
        }
    }
}

impl Book {
    fn detail(self) -> ItemDetail {
        ItemDetail {
            item: Item {
                id: self.id.clone(),
                title: titled(self.metadata.title.as_ref(), &self.name),
                year: self.metadata.year(),
                medium: Medium::Other,
                holds: Holds {
                    poster: true,
                    backdrop: false,
                    plays: false,
                },
            },
            overview: self.metadata.overview(),
            minutes: None,
            genres: Vec::new(),
            certificate: None,
            released: self.metadata.released(),
            seasons: Vec::new(),
        }
    }
}

impl Komga {
    /// What Komga answers for `path`, nothing where it holds nothing there.
    async fn found<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        what: &str,
    ) -> Result<Option<T>, Failure> {
        let response = self
            .as_admin(lemonfiber_ports::http::Method::Get, path, None)
            .await?;
        if response.status == ABSENT {
            return Ok(None);
        }
        self.endpoint.decode(&response, what).map(Some)
    }

    async fn titled(&self, id: &str) -> Result<Option<ItemDetail>, Failure> {
        if !an_id(id) {
            return Ok(None);
        }
        if let Some(series) = self
            .found::<Series>(
                &format!("{SERIES}/{id}"),
                "the series Komga holds could not be read",
            )
            .await?
        {
            return Ok(Some(series.detail()));
        }
        Ok(self
            .found::<Book>(
                &format!("{BOOKS}/{id}"),
                "the book Komga holds could not be read",
            )
            .await?
            .map(Book::detail))
    }

    async fn thumbnail(&self, path: &str) -> Result<Option<Image>, Failure> {
        let fetched = self.fetched_as_admin(path, PICTURE_MOST).await?;
        if fetched.status == ABSENT {
            return Ok(None);
        }
        if !fetched.is_success() {
            return Err(self
                .endpoint
                .refused("Komga would not answer the thumbnail"));
        }
        let served = fetched.header("content-type").map(str::to_owned);
        Ok(fetched
            .bytes
            .and_then(|bytes| picture::image(served.as_deref(), bytes)))
    }

    async fn poster(&self, id: &str) -> Result<Option<Image>, Failure> {
        if !an_id(id) {
            return Ok(None);
        }
        match self.thumbnail(&format!("{SERIES}/{id}/thumbnail")).await? {
            Some(image) => Ok(Some(image)),
            None => self.thumbnail(&format!("{BOOKS}/{id}/thumbnail")).await,
        }
    }

    async fn held(&self, most: u32) -> Result<Vec<Item>, Failure> {
        if most == 0 {
            return Ok(Vec::new());
        }
        let asked = form_encoded(&[("size", &most.to_string()), ("sort", BY_TITLE)]);
        let page: Page<Series> = self
            .read(
                &format!("{SERIES}?{asked}"),
                "the series Komga holds could not be read",
            )
            .await?;
        Ok(page
            .content
            .iter()
            .filter(|series| an_id(&series.id))
            .take(usize::try_from(most).unwrap_or(usize::MAX))
            .map(Series::item)
            .collect())
    }
}

#[async_trait]
impl Screening for Komga {
    async fn signed_in(&self, _member: &str, _device: &str) -> Result<Option<String>, Failure> {
        Err(Self::unsupported(NO_MEMBERS))
    }

    async fn title(&self, member: Option<&str>, id: &str) -> Result<Option<ItemDetail>, Failure> {
        if member.is_some() {
            return Err(Self::unsupported(NO_MEMBERS));
        }
        self.titled(id).await
    }

    async fn part_way(&self, _member: &str, _most: u32) -> Result<Vec<ItemProgress>, Failure> {
        Err(Self::unsupported(NO_MEMBERS))
    }

    async fn progressed(&self, _member: &str, _id: &str, _how_far: &HowFar) -> Result<(), Failure> {
        Err(Self::unsupported(NO_MEMBERS))
    }

    async fn sign_out(&self, _device: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_DEVICES))
    }

    async fn picture(
        &self,
        member: Option<&str>,
        id: &str,
        which: Picture,
    ) -> Result<Option<Image>, Failure> {
        if member.is_some() {
            return Err(Self::unsupported(NO_MEMBERS));
        }
        match which {
            Picture::Poster => self.poster(id).await,
            Picture::Backdrop => Ok(None),
        }
    }

    async fn holdings(&self, member: Option<&str>, most: u32) -> Result<Vec<Item>, Failure> {
        if member.is_some() {
            return Err(Self::unsupported(NO_MEMBERS));
        }
        self.held(most).await
    }

    async fn playing(&self, _member: Option<&str>) -> Result<Vec<Playback>, Failure> {
        Err(Self::unsupported(NOTHING_PLAYS))
    }
}
