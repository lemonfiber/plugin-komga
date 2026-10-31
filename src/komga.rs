//! Komga, the comics server, asked through its own API as the administrator lemonfiber
//! minted for this adapter, signed in once and carried on the session Komga opens.

use std::sync::{Arc, Mutex};

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use lemonfiber_adapter_kit::credential::Holding;
use lemonfiber_adapter_kit::endpoint::Endpoint;
use lemonfiber_ports::http::{Fetched, Http, Method, Request, Response, JSON};
use lemonfiber_ports::service::Failure;
use serde::Deserialize;

mod keys;
mod owning;
mod screening;
mod serving;

/// What every failure is blamed on.
pub const SERVICE: &str = "komga";

const HEALTH: &str = "/actuator/health";

const ME: &str = "/api/v2/users/me";

const LIBRARIES: &str = "/api/v1/libraries";

const SERIES: &str = "/api/v1/series";

const BOOKS: &str = "/api/v1/books";

const SESSION: &str = "KOMGA-SESSION";

const SET_COOKIE: &str = "set-cookie";

const COOKIE: &str = "Cookie";

const AUTHORIZATION: &str = "Authorization";

const CONTENT_TYPE: &str = "Content-Type";

const REFUSED: u16 = 401;

const ABSENT: u16 = 404;

const ID_LENGTH: usize = 13;

const NO_ADMIN: &str = "this adapter holds no administrator for Komga";

/// A client for one Komga, signed in as the administrator `email` and `password` supply,
/// holding the session it opened until Komga refuses it.
pub struct Komga {
    http: Arc<dyn Http>,
    endpoint: Endpoint,
    email: Arc<dyn Holding>,
    password: Arc<dyn Holding>,
    session: Mutex<Option<String>>,
}

#[derive(Debug, Deserialize)]
struct LibraryHeld {
    id: String,
    root: String,
}

impl Komga {
    /// The Komga reached at `base` over `http`, signed in as the administrator `email` and
    /// `password` supply.
    #[must_use]
    pub fn new(
        http: Arc<dyn Http>,
        base: impl Into<String>,
        email: Arc<dyn Holding>,
        password: Arc<dyn Holding>,
    ) -> Self {
        Self {
            endpoint: Endpoint::new(Arc::clone(&http), base, SERVICE),
            http,
            email,
            password,
            session: Mutex::new(None),
        }
    }

    /// Whether Komga says it is up, which it says to anybody.
    pub async fn answers(&self) -> bool {
        self.endpoint
            .send(&self.request(Method::Get, HEALTH, None, Vec::new()))
            .await
            .is_ok_and(|response| response.is_success())
    }

    fn minted(&self) -> Result<(String, String), Failure> {
        let held = |holding: &Arc<dyn Holding>| {
            holding
                .key()
                .map_err(|why| self.endpoint.refused(&format!("{NO_ADMIN}: {why}")))
        };
        Ok((held(&self.email)?, held(&self.password)?))
    }

    fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
        mut headers: Vec<(String, String)>,
    ) -> Request {
        if body.is_some() {
            headers.push((CONTENT_TYPE.to_owned(), JSON.to_owned()));
        }
        Request {
            method,
            url: self.endpoint.url(path),
            headers,
            body,
            pinned: None,
        }
    }

    /// The session Komga opens for `email` and `password`, or nothing where it does not
    /// sign them in.
    async fn signed_in_as(&self, email: &str, password: &str) -> Result<Option<String>, Failure> {
        let basic = STANDARD.encode(format!("{email}:{password}"));
        let response = self
            .endpoint
            .send(&self.request(
                Method::Get,
                ME,
                None,
                vec![(AUTHORIZATION.to_owned(), format!("Basic {basic}"))],
            ))
            .await?;
        if response.status == REFUSED {
            return Ok(None);
        }
        self.endpoint.expect_success(&response)?;
        session_in(&response.headers).map(Some).ok_or_else(|| {
            self.endpoint
                .refused("Komga signed the administrator in and opened no session")
        })
    }

    fn keep(&self, session: String) {
        if let Ok(mut held) = self.session.lock() {
            held.replace(session);
        }
    }

    async fn opened(&self) -> Result<String, Failure> {
        let (email, password) = self.minted()?;
        let session = self
            .signed_in_as(&email, &password)
            .await?
            .ok_or_else(|| self.endpoint.unauthorised())?;
        self.keep(session.clone());
        Ok(session)
    }

    async fn session(&self) -> Result<String, Failure> {
        let held = self.session.lock().ok().and_then(|held| held.clone());
        match held {
            Some(session) => Ok(session),
            None => self.opened().await,
        }
    }

    fn carrying(session: &str) -> Vec<(String, String)> {
        vec![(COOKIE.to_owned(), format!("{SESSION}={session}"))]
    }

    async fn as_admin(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
    ) -> Result<Response, Failure> {
        let session = self.session().await?;
        let response = self
            .endpoint
            .send(&self.request(method, path, body.clone(), Self::carrying(&session)))
            .await?;
        if response.status != REFUSED {
            return Ok(response);
        }
        let session = self.opened().await?;
        self.endpoint
            .send(&self.request(method, path, body, Self::carrying(&session)))
            .await
    }

    async fn fetched_as_admin(&self, path: &str, most: usize) -> Result<Fetched, Failure> {
        let session = self.session().await?;
        let fetched = self.fetched(path, &session, most).await?;
        if fetched.status != REFUSED {
            return Ok(fetched);
        }
        let session = self.opened().await?;
        self.fetched(path, &session, most).await
    }

    async fn fetched(&self, path: &str, session: &str, most: usize) -> Result<Fetched, Failure> {
        self.http
            .fetch(
                &self.request(Method::Get, path, None, Self::carrying(session)),
                most,
            )
            .await
            .map_err(|_| Failure::Unavailable {
                service: SERVICE.to_owned(),
            })
    }

    async fn read<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        what: &str,
    ) -> Result<T, Failure> {
        let response = self.as_admin(Method::Get, path, None).await?;
        self.endpoint.decode(&response, what)
    }

    async fn done(&self, method: Method, path: &str, body: Option<String>) -> Result<(), Failure> {
        let response = self.as_admin(method, path, body).await?;
        self.endpoint.expect_success(&response)
    }

    async fn libraries(&self) -> Result<Vec<LibraryHeld>, Failure> {
        let held: Vec<LibraryHeld> = self
            .read(LIBRARIES, "the libraries Komga holds could not be read")
            .await?;
        Ok(held
            .into_iter()
            .filter(|library| an_id(&library.id))
            .collect())
    }

    fn unsupported(detail: &str) -> Failure {
        Failure::Unsupported {
            service: SERVICE.to_owned(),
            detail: detail.to_owned(),
        }
    }
}

/// The session a response sets, where it sets one that is not empty.
fn session_in(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case(SET_COOKIE))
        .filter_map(|(_, cookie)| cookie.split(';').next())
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, value)| *name == SESSION && !value.is_empty())
        .map(|(_, value)| value.to_owned())
}

/// Whether `id` is spelled as the identifiers Komga assigns are: thirteen upper-case
/// letters and digits.
fn an_id(id: &str) -> bool {
    id.len() == ID_LENGTH
        && id
            .bytes()
            .all(|mark| mark.is_ascii_uppercase() || mark.is_ascii_digit())
}

#[cfg(test)]
mod tests;
