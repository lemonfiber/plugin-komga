//! Looking over Komga's libraries again, and reading one series or book afresh. Komga
//! holds neither films nor television, keeps no list of proxies or origins anybody but its
//! own configuration sets, and is never the household's identity source, so the rest is
//! unsupported.

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::media::Kind;
use lemonfiber_ports::service::{
    Applied, Declared, Declaring, Encrypting, Failure, Fronted, Library, SeriesHeld, Upkeep,
};

use super::{an_id, Komga, ABSENT, BOOKS, LIBRARIES, SERIES};

const NO_KINDS: &str = "Komga holds comics, never films or television";

const NO_PROXIES: &str = "Komga holds no list of addresses trusted to name the client its API sets";

const NO_ORIGINS: &str = "Komga holds no list of origins a browser may read it from its API sets";

const NO_RESTART: &str = "Komga is not asked to start again by this adapter";

const NO_GUARD: &str = "Komga serves no member through a guard of lemonfiber's";

const NO_TLS: &str = "Komga serves no internal TLS port";

const NOTHING_THERE: &str = "Komga holds no series or book under that id";

#[async_trait]
impl Library for Komga {
    async fn has_item(&self, _kind: Kind, _term: &str) -> Result<bool, Failure> {
        Err(Self::unsupported(NO_KINDS))
    }

    async fn rescan(&self) -> Result<(), Failure> {
        self.furnished().await?;
        for library in self.libraries().await? {
            self.done(
                Method::Post,
                &format!("{LIBRARIES}/{}/scan", library.id),
                None,
            )
            .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Upkeep for Komga {
    async fn series_held(&self, _most: u32) -> Result<Vec<SeriesHeld>, Failure> {
        Err(Self::unsupported(NO_KINDS))
    }

    async fn refresh(&self, id: &str) -> Result<(), Failure> {
        if !an_id(id) {
            return Err(self.endpoint.refused(NOTHING_THERE));
        }
        for held in [SERIES, BOOKS] {
            let response = self
                .as_admin(Method::Post, &format!("{held}/{id}/metadata/refresh"), None)
                .await?;
            if response.status != ABSENT {
                return self.endpoint.expect_success(&response);
            }
        }
        Err(self.endpoint.refused(NOTHING_THERE))
    }
}

#[async_trait]
impl Fronted for Komga {
    async fn known_proxies(&self) -> Result<Vec<String>, Failure> {
        Err(Self::unsupported(NO_PROXIES))
    }

    async fn trust_only(&self, _address: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_PROXIES))
    }

    async fn allowed_origins(&self) -> Result<Vec<String>, Failure> {
        Err(Self::unsupported(NO_ORIGINS))
    }

    async fn allow_only(&self, _origin: &str) -> Result<(), Failure> {
        Err(Self::unsupported(NO_ORIGINS))
    }

    async fn restart(&self) -> Result<(), Failure> {
        Err(Self::unsupported(NO_RESTART))
    }
}

#[async_trait]
impl Declaring for Komga {
    async fn declared(&self) -> Result<Declared, Failure> {
        Err(Self::unsupported(NO_GUARD))
    }
}

#[async_trait]
impl Encrypting for Komga {
    async fn serve_tls(&self, _path: &str, _pin: &str) -> Result<Applied, Failure> {
        Err(Self::unsupported(NO_TLS))
    }
}
