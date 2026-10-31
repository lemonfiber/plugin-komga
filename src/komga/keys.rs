//! The API keys Komga holds for the stack's services, each filed under an app's name and
//! made for the administrator. Komga shows a key once, when it is minted, so the keys filed
//! under a name are unsupported; it refuses two keys under one name, so a second is filed
//! as `<app> #2`, a third as `<app> #3`, and so on. A key is found again by what Komga
//! records of the request it signed in: the key's id, beside the user agent that request
//! carried.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use lemonfiber_adapter_kit::credential::spelled_as_a_key;
use lemonfiber_adapter_kit::endpoint::form_encoded;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{AppKeys, Dated, Failure};
use serde::Deserialize;

use super::{an_id, Komga, ABSENT, ME};

const KEYS: &str = "/api/v2/users/me/api-keys";

const ACTIVITY: &str = "/api/v2/users/me/authentication-activity";

const ACTIVITY_MOST: &str = "50";

const API_KEY: &str = "X-API-Key";

const USER_AGENT: &str = "User-Agent";

const SOURCE: &str = "ApiKey";

const SHOWN_ONCE: &str = "Komga shows a key only once, when it is minted";

const NOT_HELD: &str = "Komga holds no key that one names";

const FIRST_SUFFIX: u32 = 2;

static ASKED: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Key {
    id: String,
    comment: String,
    #[serde(default)]
    created_date: Option<String>,
}

#[derive(Deserialize)]
struct Minted {
    id: String,
    key: String,
}

#[derive(Deserialize)]
struct Me {
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Activity {
    #[serde(default)]
    date_time: Option<String>,
}

#[derive(Deserialize)]
struct Activities {
    content: Vec<Signed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Signed {
    #[serde(default)]
    api_key_id: Option<String>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

/// Whether `comment` files a key under `app`: `app` itself, or `app #<n>`.
pub(super) fn filed_under(comment: &str, app: &str) -> bool {
    comment == app
        || comment
            .strip_prefix(app)
            .and_then(|rest| rest.strip_prefix(" #"))
            .is_some_and(|number| number.parse::<u32>().is_ok())
}

/// The name the next key minted for `app` is filed under, beside the `held` names.
pub(super) fn next_name(app: &str, held: &[String]) -> String {
    if !held.iter().any(|comment| comment == app) {
        return app.to_owned();
    }
    let mut number = FIRST_SUFFIX;
    loop {
        let name = format!("{app} #{number}");
        if !held.contains(&name) {
            return name;
        }
        number = number.saturating_add(1);
    }
}

/// A user agent no other request of this process carries.
fn marker() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    format!(
        "lemonfiber-komga-adapter/{now}-{}",
        ASKED.fetch_add(1, Ordering::Relaxed)
    )
}

impl Komga {
    async fn every_key(&self) -> Result<Vec<Key>, Failure> {
        self.read(KEYS, "the keys Komga holds could not be read")
            .await
    }

    async fn listed(&self, app: &str) -> Result<Vec<Key>, Failure> {
        let mut filed: Vec<Key> = self
            .every_key()
            .await?
            .into_iter()
            .filter(|key| filed_under(&key.comment, app))
            .collect();
        filed.sort_by(|one, other| one.created_date.cmp(&other.created_date));
        Ok(filed)
    }

    fn a_key(&self, key: &str) -> Result<(), Failure> {
        if spelled_as_a_key(key) {
            Ok(())
        } else {
            Err(self.endpoint.refused("that is not spelled as a key is"))
        }
    }

    /// Ask who `key` signs in as, carrying `agent`.
    async fn signed_in_with(&self, key: &str, agent: Option<&str>) -> Result<(), Failure> {
        let mut headers = vec![(API_KEY.to_owned(), key.to_owned())];
        if let Some(agent) = agent {
            headers.push((USER_AGENT.to_owned(), agent.to_owned()));
        }
        let response = self
            .endpoint
            .send(&self.request(Method::Get, ME, None, headers))
            .await?;
        self.endpoint.expect_success(&response)
    }

    async fn last_used(&self, me: &str, key: &str) -> Result<Option<String>, Failure> {
        let asked = form_encoded(&[("apikey_id", key)]);
        let response = self
            .as_admin(
                Method::Get,
                &format!("/api/v2/users/{me}/authentication-activity/latest?{asked}"),
                None,
            )
            .await?;
        if response.status == ABSENT {
            return Ok(None);
        }
        let activity: Activity = self
            .endpoint
            .decode(&response, "when a key was last used could not be read")?;
        Ok(activity.date_time)
    }
}

#[async_trait]
impl AppKeys for Komga {
    async fn filed_as(&self, _app: &str) -> Result<Vec<String>, Failure> {
        Err(Self::unsupported(SHOWN_ONCE))
    }

    async fn dated(&self, app: &str) -> Result<Vec<Dated>, Failure> {
        let me: Me = self
            .read(
                ME,
                "the account this adapter signed in as could not be read",
            )
            .await?;
        let mut dated = Vec::new();
        for key in self.listed(app).await? {
            let last_used = if an_id(&key.id) {
                self.last_used(&me.id, &key.id).await?
            } else {
                None
            };
            dated.push(Dated {
                created: key.created_date,
                last_used,
            });
        }
        Ok(dated)
    }

    async fn mint(&self, app: &str) -> Result<String, Failure> {
        let held = self.every_key().await?;
        let comments: Vec<String> = held.iter().map(|key| key.comment.clone()).collect();
        let before: Vec<String> = held.into_iter().map(|key| key.id).collect();
        let asked = serde_json::json!({ "comment": next_name(app, &comments) }).to_string();
        let response = self.as_admin(Method::Post, KEYS, Some(asked)).await?;
        let minted: Minted = self
            .endpoint
            .decode(&response, "the key Komga minted could not be read")?;
        let new: Vec<String> = self
            .listed(app)
            .await?
            .into_iter()
            .map(|key| key.id)
            .filter(|id| !before.contains(id))
            .collect();
        if new != [minted.id.as_str()] || !spelled_as_a_key(&minted.key) {
            return Err(self.endpoint.refused(&format!(
                "a key was minted as {app} and the key list does not show it as the one new key"
            )));
        }
        Ok(minted.key)
    }

    async fn answers_to(&self, key: &str) -> Result<(), Failure> {
        self.a_key(key)?;
        self.signed_in_with(key, None).await
    }

    async fn revoke(&self, key: &str) -> Result<(), Failure> {
        self.a_key(key)?;
        let agent = marker();
        match self.signed_in_with(key, Some(&agent)).await {
            Ok(()) => {}
            Err(unavailable @ Failure::Unavailable { .. }) => return Err(unavailable),
            Err(_) => return Err(self.endpoint.refused(NOT_HELD)),
        }
        let asked = form_encoded(&[("size", ACTIVITY_MOST), ("sort", "dateTime,desc")]);
        let activities: Activities = self
            .read(
                &format!("{ACTIVITY}?{asked}"),
                "what Komga recorded of the key's sign-in could not be read",
            )
            .await?;
        let id = activities
            .content
            .into_iter()
            .find(|signed| {
                signed.user_agent.as_deref() == Some(agent.as_str())
                    && signed.source.as_deref() == Some(SOURCE)
            })
            .and_then(|signed| signed.api_key_id)
            .filter(|id| an_id(id))
            .ok_or_else(|| self.endpoint.refused(NOT_HELD))?;
        if !self.every_key().await?.iter().any(|held| held.id == id) {
            return Err(self.endpoint.refused(NOT_HELD));
        }
        self.done(Method::Delete, &format!("{KEYS}/{id}"), None)
            .await
    }
}
