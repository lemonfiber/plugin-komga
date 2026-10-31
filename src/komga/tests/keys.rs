use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::{Http, Method, Request, Response, Unreachable};
use lemonfiber_ports::service::{AppKeys as _, Dated};

use super::super::keys::{filed_under, next_name};
use super::super::Komga;
use super::{asked, body_of, komga, refused_saying, signed_in_and, unsupported, BASE};

const KEY: &str = "abcdefghijklmnopqrstuvwxyz012345";

const MINTED: &str = r#"{"id":"0S31NY0GGVJMB","userId":"0S31NWRD0VVA9","key":"abcdefghijklmnopqrstuvwxyz012345","comment":"dashboard"}"#;

const HELD: &str =
    r#"[{"id":"0S31NY0GGVJMB","comment":"dashboard","createdDate":"2026-10-30T23:56:50Z"}]"#;

/// A Komga that records what a key's sign-in carried as its user agent, and answers its
/// authentication activity with it, as Komga records each request a key signs in.
struct Recording {
    fake: Arc<Fake>,
    agent: Mutex<Option<String>>,
    key_id: &'static str,
    source: &'static str,
}

#[async_trait]
impl Http for Recording {
    async fn send(&self, request: &Request) -> Result<Response, Unreachable> {
        if request.headers.iter().any(|(name, _)| name == "X-API-Key") {
            if let Ok(mut agent) = self.agent.lock() {
                *agent = request
                    .headers
                    .iter()
                    .find(|(name, _)| name == "User-Agent")
                    .map(|(_, value)| value.clone());
            }
        }
        if request.url.contains("/authentication-activity?") {
            let agent = self.agent.lock().ok().and_then(|agent| agent.clone());
            let body = serde_json::json!({ "content": [
                { "apiKeyId": null, "userAgent": "curl/8", "source": "Password" },
                { "apiKeyId": self.key_id, "userAgent": agent, "source": self.source },
            ]})
            .to_string();
            return Ok(Response {
                status: 200,
                headers: Vec::new(),
                body,
            });
        }
        self.fake.send(request).await
    }
}

fn recording(key_id: &'static str, source: &'static str, held: &'static str) -> Arc<Recording> {
    Arc::new(Recording {
        fake: signed_in_and(vec![
            (
                Method::Delete,
                "/api/v2/users/me/api-keys/",
                Answer::reply(204, ""),
            ),
            (
                Method::Get,
                "/api/v2/users/me/api-keys",
                Answer::reply(200, held),
            ),
        ]),
        agent: Mutex::new(None),
        key_id,
        source,
    })
}

fn over(http: &Arc<Recording>) -> Komga {
    Komga::new(
        http.clone(),
        BASE,
        Arc::new(super::EMAIL.to_owned()),
        Arc::new(super::PASSWORD.to_owned()),
    )
}

#[test]
fn a_key_is_filed_under_its_app_alone_or_numbered() {
    assert!(filed_under("dashboard", "dashboard"));
    assert!(filed_under("dashboard #2", "dashboard"));
    for not in [
        "dashboards",
        "dashboard #",
        "dashboard #two",
        "dashboard 2",
        "other",
    ] {
        assert!(!filed_under(not, "dashboard"), "{not}");
    }
}

#[test]
fn the_next_key_takes_the_app_s_name_or_the_first_number_free() {
    let held = |names: &[&str]| {
        names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(next_name("dashboard", &held(&[])), "dashboard");
    assert_eq!(next_name("dashboard", &held(&["other"])), "dashboard");
    assert_eq!(
        next_name("dashboard", &held(&["dashboard"])),
        "dashboard #2"
    );
    assert_eq!(
        next_name(
            "dashboard",
            &held(&["dashboard", "dashboard #2", "dashboard #4"])
        ),
        "dashboard #3"
    );
}

#[tokio::test]
async fn the_keys_filed_under_a_name_are_unsupported_since_komga_shows_each_once() {
    let fake = Fake::silent();
    assert!(unsupported(komga(&fake).filed_as("dashboard").await));
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn a_key_is_minted_under_the_app_s_name_and_held_to_the_one_new_key_listed() {
    let fake = Fake::by_route_in_turn(vec![
        (
            Method::Get,
            "/api/v2/users/me/api-keys",
            vec![Answer::reply(200, "[]"), Answer::reply(200, HELD)],
        ),
        (
            Method::Post,
            "/api/v2/users/me/api-keys",
            vec![Answer::reply(200, MINTED)],
        ),
        (Method::Get, "/api/v2/users/me", vec![super::signed_in()]),
    ]);
    assert_eq!(
        komga(&fake).mint("dashboard").await.ok().as_deref(),
        Some(KEY)
    );
    assert_eq!(
        body_of(&fake, "/api/v2/users/me/api-keys"),
        Some(serde_json::json!({ "comment": "dashboard" }))
    );
}

#[tokio::test]
async fn a_second_key_for_one_app_is_numbered() {
    let fake = Fake::by_route_in_turn(vec![
        (
            Method::Get,
            "/api/v2/users/me/api-keys",
            vec![
                Answer::reply(200, HELD),
                Answer::reply(
                    200,
                    r#"[{"id":"0S31NY0GGVJMB","comment":"dashboard"},{"id":"0S31NY0GGVJMC","comment":"dashboard #2"}]"#,
                ),
            ],
        ),
        (
            Method::Post,
            "/api/v2/users/me/api-keys",
            vec![Answer::reply(
                200,
                r#"{"id":"0S31NY0GGVJMC","key":"abcdefghijklmnopqrstuvwxyz012345"}"#,
            )],
        ),
        (Method::Get, "/api/v2/users/me", vec![super::signed_in()]),
    ]);
    assert!(komga(&fake).mint("dashboard").await.is_ok());
    assert_eq!(
        body_of(&fake, "/api/v2/users/me/api-keys"),
        Some(serde_json::json!({ "comment": "dashboard #2" }))
    );
}

#[tokio::test]
async fn a_minted_key_the_list_does_not_show_or_that_is_not_one_is_refused() {
    for (listed, minted) in [
        ("[]", MINTED),
        (HELD, r#"{"id":"0S31NY0GGVJMB","key":"not a key"}"#),
    ] {
        let fake = Fake::by_route_in_turn(vec![
            (
                Method::Get,
                "/api/v2/users/me/api-keys",
                vec![Answer::reply(200, "[]"), Answer::reply(200, listed)],
            ),
            (
                Method::Post,
                "/api/v2/users/me/api-keys",
                vec![Answer::reply(200, minted)],
            ),
            (Method::Get, "/api/v2/users/me", vec![super::signed_in()]),
        ]);
        assert!(refused_saying(
            komga(&fake).mint("dashboard").await,
            "does not show it as the one new key"
        ));
    }
}

#[tokio::test]
async fn each_key_filed_under_an_app_is_dated_by_when_it_was_made_and_last_signed_in() {
    let fake = signed_in_and(vec![
        (
            Method::Get,
            "/authentication-activity/latest?apikey_id=0S31NY0GGVJMB",
            Answer::reply(200, r#"{"dateTime":"2026-10-31T01:02:03Z"}"#),
        ),
        (
            Method::Get,
            "/authentication-activity/latest?apikey_id=0S31NY0GGVJMC",
            Answer::reply(404, ""),
        ),
        (
            Method::Get,
            "/api/v2/users/me/api-keys",
            Answer::reply(
                200,
                r#"[{"id":"0S31NY0GGVJMB","comment":"dashboard","createdDate":"2026-10-02T00:00:00Z"},
                    {"id":"0S31NY0GGVJMC","comment":"dashboard #2","createdDate":"2026-10-01T00:00:00Z"},
                    {"id":"bad","comment":"dashboard #3"},
                    {"id":"0S31NY0GGVJMD","comment":"other"}]"#,
            ),
        ),
    ]);
    assert_eq!(
        komga(&fake).dated("dashboard").await.ok(),
        Some(vec![
            Dated {
                created: None,
                last_used: None,
            },
            Dated {
                created: Some("2026-10-01T00:00:00Z".to_owned()),
                last_used: None,
            },
            Dated {
                created: Some("2026-10-02T00:00:00Z".to_owned()),
                last_used: Some("2026-10-31T01:02:03Z".to_owned()),
            },
        ])
    );
    assert!(asked(&fake).contains(&(
        Method::Get,
        "/api/v2/users/0S31NWRD0VVA9/authentication-activity/latest?apikey_id=0S31NY0GGVJMB"
            .to_owned()
    )));
}

#[tokio::test]
async fn a_key_is_answered_to_where_komga_signs_it_in() {
    let fake = Fake::by_route(vec![(
        Method::Get,
        "/api/v2/users/me",
        Answer::reply(200, "{}"),
    )]);
    assert!(komga(&fake).answers_to(KEY).await.is_ok());
    assert_eq!(
        super::headers_of(&fake, Method::Get, "/api/v2/users/me"),
        [("X-API-Key".to_owned(), KEY.to_owned())]
    );
    let refusing = Fake::by_route(vec![(
        Method::Get,
        "/api/v2/users/me",
        Answer::reply(401, ""),
    )]);
    assert!(komga(&refusing).answers_to(KEY).await.is_err());
}

#[tokio::test]
async fn what_is_not_spelled_as_a_key_is_refused_and_nothing_is_asked() {
    let fake = Fake::silent();
    let komga = komga(&fake);
    assert!(refused_saying(
        komga.answers_to("not a key").await,
        "not spelled"
    ));
    assert!(refused_saying(
        komga.revoke("not a key").await,
        "not spelled"
    ));
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn a_key_is_revoked_by_the_id_komga_recorded_for_its_sign_in() {
    let http = recording("0S31NY0GGVJMB", "ApiKey", HELD);
    assert!(over(&http).revoke(KEY).await.is_ok());
    let asked = asked(&http.fake);
    assert!(asked.contains(&(
        Method::Delete,
        "/api/v2/users/me/api-keys/0S31NY0GGVJMB".to_owned()
    )));
    let agent = http.agent.lock().ok().and_then(|agent| agent.clone());
    assert!(agent.is_some_and(|agent| agent.starts_with("lemonfiber-komga-adapter/")));
}

#[tokio::test]
async fn a_key_komga_does_not_sign_in_or_does_not_record_is_not_revoked() {
    let refusing = Fake::by_route(vec![(
        Method::Get,
        "/api/v2/users/me",
        Answer::reply(401, ""),
    )]);
    assert!(refused_saying(komga(&refusing).revoke(KEY).await, "no key"));
    for http in [
        recording("0S31NY0GGVJMB", "Password", HELD),
        recording("bad", "ApiKey", HELD),
        recording("0S31NY0GGVJMZ", "ApiKey", HELD),
    ] {
        assert!(refused_saying(over(&http).revoke(KEY).await, "no key"));
        assert!(!http
            .fake
            .requests()
            .iter()
            .any(|one| one.method == Method::Delete));
    }
}

#[tokio::test]
async fn a_komga_that_does_not_answer_dates_and_answers_to_no_key() {
    let silent = Fake::silent();
    assert!(komga(&silent).answers_to(KEY).await.is_err());
    assert!(matches!(
        komga(&silent).revoke(KEY).await,
        Err(lemonfiber_ports::service::Failure::Unavailable { .. })
    ));
    let keys_unread = signed_in_and(vec![(
        Method::Get,
        "/api/v2/users/me/api-keys",
        Answer::Silent,
    )]);
    assert!(komga(&keys_unread).dated("dashboard").await.is_err());
    let activity_unread = signed_in_and(vec![
        (
            Method::Get,
            "/authentication-activity/latest",
            Answer::Silent,
        ),
        (
            Method::Get,
            "/api/v2/users/me/api-keys",
            Answer::reply(200, HELD),
        ),
    ]);
    assert!(komga(&activity_unread).dated("dashboard").await.is_err());
}
