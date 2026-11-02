//! Who holds Komga's administrator, and claiming it. A fresh Komga holds no account, and
//! the first caller to ask makes the administrator: claiming it makes that administrator
//! the one lemonfiber minted.

use async_trait::async_trait;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Claim, Failure, Owner, Ownership};
use serde::Deserialize;

use super::Komga;

const CLAIM: &str = "/api/v1/claim";

const EMAIL: &str = "X-Komga-Email";

const PASSWORD: &str = "X-Komga-Password";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claimed {
    is_claimed: bool,
}

impl Komga {
    /// Whether Komga holds an administrator yet, or nothing where it does not answer.
    async fn claimed(&self) -> Result<Option<bool>, Failure> {
        let Ok(response) = self
            .endpoint
            .send(&self.request(Method::Get, CLAIM, None, Vec::new()))
            .await
        else {
            return Ok(None);
        };
        if response.status >= 500 {
            return Ok(None);
        }
        let claimed: Claimed = self.endpoint.decode(
            &response,
            "whether Komga holds an administrator could not be read",
        )?;
        Ok(Some(claimed.is_claimed))
    }

    /// `Ours` where the minted administrator signs in, keeping its session, and
    /// `Theirs` otherwise.
    async fn held_by(&self) -> Result<Owner, Failure> {
        let (email, password) = self.minted()?;
        match self.signed_in_as(&email, &password).await? {
            Some(session) => {
                self.keep(session);
                Ok(Owner::Ours)
            }
            None => Ok(Owner::Theirs),
        }
    }

    /// Claim Komga where nobody holds it, and say who holds it then.
    async fn claiming(&self) -> Result<Owner, Failure> {
        match self.claimed().await? {
            None => return Ok(Owner::Settling),
            Some(true) => return self.held_by().await,
            Some(false) => {}
        }
        let (email, password) = self.minted()?;
        let answer = self
            .endpoint
            .send(&self.request(
                Method::Post,
                CLAIM,
                None,
                vec![(EMAIL.to_owned(), email), (PASSWORD.to_owned(), password)],
            ))
            .await?;
        if !answer.is_success() {
            return match self.claimed().await? {
                Some(true) => self.held_by().await,
                _ => Err(self.endpoint.refusal(&answer)),
            };
        }
        let owner = self.held_by().await?;
        if owner != Owner::Ours {
            return Err(self.endpoint.refused(
                "Komga took the administrator lemonfiber minted and does not sign it in",
            ));
        }
        Ok(Owner::Ours)
    }
}

#[async_trait]
impl Ownership for Komga {
    async fn owner(&self) -> Result<Owner, Failure> {
        match self.claimed().await? {
            None => Ok(Owner::Settling),
            Some(false) => Ok(Owner::Open),
            Some(true) => self.held_by().await,
        }
    }

    async fn claim(&self) -> Result<Claim, Failure> {
        self.claiming().await.map(Claim::Held)
    }
}
