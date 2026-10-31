//! What this adapter says it is, and the kit that serves its contracts.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::{media, owner};

use crate::komga::Komga;

/// Where Komga answers, by its service id and the port it listens on.
pub const ADDRESS: &str = "http://komga:25600";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "Komga";

/// The Komga release this adapter's recordings were taken from.
pub const RELEASE: &str = "1.26.3";

/// The digest of the image that release's recordings were taken from.
pub const DIGEST: &str = "sha256:6c2a967bbe9acefd05933b2eb498f34afe96a83c6f7f8ab0acb512a1bb3ab50f";

/// What this adapter says it is: the contracts it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![
            lemonfiber_contract::spoken(owner::claim::CAPABILITY, owner::claim::MAJOR),
            lemonfiber_contract::spoken(media::serve::CAPABILITY, media::serve::MAJOR),
        ],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contracts, served from `komga` under `key`, ready while it answers.
#[must_use]
pub fn kit(key: String, komga: Arc<Komga>) -> Kit {
    let asked = Arc::clone(&komga);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(owner::claim::served(Arc::clone(&komga)))
        .serving(media::serve::served(komga))
}

#[cfg(test)]
mod tests;
