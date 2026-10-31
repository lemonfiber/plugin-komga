use std::sync::Arc;

use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Failure, Owner, Ownership as _};

use super::{an_id, session_in, Komga};

mod keys;
mod screening;
mod serving;

const BASE: &str = "http://komga:25600";

const EMAIL: &str = "0123456789abcdef0123456789abcdef@komga.invalid";

const PASSWORD: &str = "fedcba9876543210fedcba9876543210";

const SESSION: &str = "the-session";

const ME: &str =
    r#"{"id":"0S31NWRD0VVA9","email":"0123456789abcdef0123456789abcdef@komga.invalid"}"#;

const UNCLAIMED: &str = r#"{"isClaimed":false}"#;

const CLAIMED: &str = r#"{"isClaimed":true}"#;

const LIBRARIES: &str = r#"[{"id":"0S31NXXY4VJJX","root":"/data/media/manga"},{"id":"not-an-id","root":"/data/media/x"},{"id":"0S31NXXY4VJJY","root":"/data/media/other"}]"#;

struct Missing;

impl lemonfiber_adapter_kit::credential::Holding for Missing {
    fn key(&self) -> Result<String, String> {
        Err("upstream.json could not be read: not found".to_owned())
    }
}

fn signed_in() -> Answer {
    Answer::Headed {
        status: 200,
        headers: vec![(
            "Set-Cookie".to_owned(),
            format!("KOMGA-SESSION={SESSION}; Path=/; HttpOnly; SameSite=Lax"),
        )],
        body: ME.to_owned(),
    }
}

fn komga(fake: &Arc<Fake>) -> Komga {
    Komga::new(
        fake.clone(),
        BASE,
        Arc::new(EMAIL.to_owned()),
        Arc::new(PASSWORD.to_owned()),
    )
}

/// `routes`, then the administrator's sign-in answering with a session, last because
/// every path under it holds its fragment.
fn signed_in_and(mut routes: Vec<(Method, &'static str, Answer)>) -> Arc<Fake> {
    routes.push((Method::Get, "/api/v2/users/me", signed_in()));
    Fake::by_route(routes)
}

fn asked(fake: &Fake) -> Vec<(Method, String)> {
    fake.requests()
        .into_iter()
        .map(|one| (one.method, one.url.trim_start_matches(BASE).to_owned()))
        .collect()
}

fn body_of(fake: &Fake, path: &str) -> Option<serde_json::Value> {
    fake.requests()
        .into_iter()
        .find(|one| one.url.ends_with(path) && one.body.is_some())
        .and_then(|one| one.body)
        .and_then(|body| serde_json::from_str(&body).ok())
}

fn headers_of(fake: &Fake, method: Method, path: &str) -> Vec<(String, String)> {
    fake.requests()
        .into_iter()
        .find(|one| one.method == method && one.url.trim_start_matches(BASE) == path)
        .map(|one| one.headers)
        .unwrap_or_default()
}

fn basic() -> (String, String) {
    (
        "Authorization".to_owned(),
        "Basic MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWZAa29tZ2EuaW52YWxpZDpmZWRjYmE5ODc2NTQzMjEwZmVkY2JhOTg3NjU0MzIxMA=="
            .to_owned(),
    )
}

fn carried() -> (String, String) {
    ("Cookie".to_owned(), format!("KOMGA-SESSION={SESSION}"))
}

fn refused_saying(outcome: Result<impl std::fmt::Debug, Failure>, said: &str) -> bool {
    matches!(outcome, Err(Failure::Refused { detail, .. }) if detail.contains(said))
}

fn unsupported(outcome: Result<impl std::fmt::Debug, Failure>) -> bool {
    matches!(outcome, Err(Failure::Unsupported { service, .. }) if service == "komga")
}

#[tokio::test]
async fn it_answers_while_komga_says_it_is_up() {
    let up = Fake::always(Answer::reply(200, r#"{"status":"UP"}"#));
    assert!(komga(&up).answers().await);
    assert_eq!(asked(&up), [(Method::Get, "/actuator/health".to_owned())]);
    assert!(headers_of(&up, Method::Get, "/actuator/health").is_empty());
    for down in [Fake::silent(), Fake::always(Answer::reply(503, ""))] {
        assert!(!komga(&down).answers().await);
    }
}

#[tokio::test]
async fn one_session_is_opened_as_the_administrator_and_carried_on_every_read_after() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/libraries",
        Answer::reply(200, "[]"),
    )]);
    let komga = komga(&fake);
    assert!(komga.libraries().await.is_ok());
    assert!(komga.libraries().await.is_ok());
    assert_eq!(
        asked(&fake),
        [
            (Method::Get, "/api/v2/users/me".to_owned()),
            (Method::Get, "/api/v1/libraries".to_owned()),
            (Method::Get, "/api/v1/libraries".to_owned()),
        ]
    );
    assert_eq!(
        headers_of(&fake, Method::Get, "/api/v2/users/me"),
        [basic()]
    );
    assert!(fake
        .requests()
        .iter()
        .filter(|one| one.url.ends_with("/libraries"))
        .all(|one| one.headers == [carried()]));
}

#[tokio::test]
async fn a_session_komga_refuses_is_opened_again_once() {
    let fake = Fake::by_route_in_turn(vec![
        (
            Method::Get,
            "/api/v1/libraries",
            vec![Answer::reply(401, ""), Answer::reply(200, LIBRARIES)],
        ),
        (Method::Get, "/api/v2/users/me", vec![signed_in()]),
    ]);
    let held = komga(&fake).libraries().await;
    assert_eq!(
        held.map(|libraries| libraries
            .into_iter()
            .map(|library| (library.id, library.root))
            .collect::<Vec<_>>())
            .ok(),
        Some(vec![
            ("0S31NXXY4VJJX".to_owned(), "/data/media/manga".to_owned()),
            ("0S31NXXY4VJJY".to_owned(), "/data/media/other".to_owned()),
        ])
    );
    assert_eq!(
        asked(&fake),
        [
            (Method::Get, "/api/v2/users/me".to_owned()),
            (Method::Get, "/api/v1/libraries".to_owned()),
            (Method::Get, "/api/v2/users/me".to_owned()),
            (Method::Get, "/api/v1/libraries".to_owned()),
        ]
    );
}

#[tokio::test]
async fn an_administrator_komga_will_not_sign_in_is_refused_as_unauthorised() {
    let fake = Fake::by_route(vec![(
        Method::Get,
        "/api/v2/users/me",
        Answer::reply(401, ""),
    )]);
    assert!(matches!(
        komga(&fake).libraries().await,
        Err(Failure::Unauthorised { .. })
    ));
    assert_eq!(asked(&fake), [(Method::Get, "/api/v2/users/me".to_owned())]);
}

#[tokio::test]
async fn a_sign_in_komga_answers_without_a_session_or_with_an_error_is_refused() {
    let sessionless = Fake::by_route(vec![(
        Method::Get,
        "/api/v2/users/me",
        Answer::reply(200, ME),
    )]);
    assert!(refused_saying(
        komga(&sessionless).libraries().await,
        "opened no session"
    ));
    let failing = Fake::by_route(vec![(
        Method::Get,
        "/api/v2/users/me",
        Answer::reply(500, "down"),
    )]);
    assert!(komga(&failing).libraries().await.is_err());
}

#[tokio::test]
async fn without_the_minted_administrator_nothing_is_asked() {
    for (email, password) in [
        (
            Arc::new(Missing) as Arc<dyn lemonfiber_adapter_kit::credential::Holding>,
            Arc::new(PASSWORD.to_owned()) as Arc<dyn lemonfiber_adapter_kit::credential::Holding>,
        ),
        (Arc::new(EMAIL.to_owned()), Arc::new(Missing)),
    ] {
        let fake = Fake::silent();
        let komga = Komga::new(fake.clone(), BASE, email, password);
        assert!(refused_saying(
            komga.libraries().await,
            "this adapter holds no administrator for Komga"
        ));
        assert!(fake.requests().is_empty());
    }
}

#[test]
fn only_the_session_cookie_holding_a_value_is_taken() {
    let set = |cookie: &str| vec![("set-cookie".to_owned(), cookie.to_owned())];
    assert_eq!(
        session_in(&set("KOMGA-SESSION=abc; Path=/")).as_deref(),
        Some("abc")
    );
    assert_eq!(session_in(&set("KOMGA-SESSION=; Path=/")), None);
    assert_eq!(session_in(&set("OTHER=abc")), None);
    assert_eq!(session_in(&set("no-pair-here")), None);
    assert_eq!(
        session_in(&[("x-other".to_owned(), "KOMGA-SESSION=abc".to_owned())]),
        None
    );
}

#[test]
fn an_id_is_thirteen_upper_case_letters_and_digits() {
    assert!(an_id("0S31NWRD0VVA9"));
    for not in [
        "",
        "0S31NWRD0VVA",
        "0S31NWRD0VVA90",
        "0s31nwrd0vva9",
        "0S31NWRD0VV/9",
    ] {
        assert!(!an_id(not), "{not}");
    }
}

#[tokio::test]
async fn a_komga_not_answering_yet_is_settling() {
    for fake in [
        Fake::silent(),
        Fake::by_route(vec![(Method::Get, "/api/v1/claim", Answer::reply(503, ""))]),
    ] {
        let komga = komga(&fake);
        assert_eq!(komga.owner().await.ok(), Some(Owner::Settling));
        assert_eq!(komga.claim().await.ok(), Some(Owner::Settling));
        assert!(fake.requests().iter().all(|one| one.method == Method::Get));
    }
}

#[tokio::test]
async fn an_unclaimed_komga_is_open() {
    let fake = Fake::by_route(vec![(
        Method::Get,
        "/api/v1/claim",
        Answer::reply(200, UNCLAIMED),
    )]);
    assert_eq!(komga(&fake).owner().await.ok(), Some(Owner::Open));
    assert_eq!(asked(&fake), [(Method::Get, "/api/v1/claim".to_owned())]);
    assert!(headers_of(&fake, Method::Get, "/api/v1/claim").is_empty());
}

#[tokio::test]
async fn a_claimed_komga_is_ours_where_the_minted_administrator_signs_in_and_theirs_otherwise() {
    let ours = signed_in_and(vec![(
        Method::Get,
        "/api/v1/claim",
        Answer::reply(200, CLAIMED),
    )]);
    assert_eq!(komga(&ours).owner().await.ok(), Some(Owner::Ours));
    let theirs = Fake::by_route(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, CLAIMED)),
        (Method::Get, "/api/v2/users/me", Answer::reply(401, "")),
    ]);
    assert_eq!(komga(&theirs).owner().await.ok(), Some(Owner::Theirs));
}

#[tokio::test]
async fn a_claim_answer_that_is_not_one_is_refused() {
    let fake = Fake::by_route(vec![(
        Method::Get,
        "/api/v1/claim",
        Answer::reply(200, "<html>"),
    )]);
    assert!(komga(&fake).owner().await.is_err());
}

#[tokio::test]
async fn a_part_already_held_is_answered_as_it_is_and_never_claimed_again() {
    let ours = signed_in_and(vec![(
        Method::Get,
        "/api/v1/claim",
        Answer::reply(200, CLAIMED),
    )]);
    assert_eq!(komga(&ours).claim().await.ok(), Some(Owner::Ours));
    let theirs = Fake::by_route(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, CLAIMED)),
        (Method::Get, "/api/v2/users/me", Answer::reply(401, "")),
    ]);
    assert_eq!(komga(&theirs).claim().await.ok(), Some(Owner::Theirs));
    for fake in [ours, theirs] {
        assert!(fake.requests().iter().all(|one| one.method == Method::Get));
    }
}

#[tokio::test]
async fn an_open_komga_is_claimed_with_the_minted_administrator_and_given_the_comics_library() {
    let fake = signed_in_and(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, UNCLAIMED)),
        (Method::Post, "/api/v1/claim", Answer::reply(200, ME)),
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(200, LIBRARIES),
        ),
        (Method::Post, "/api/v1/libraries", Answer::reply(200, "{}")),
    ]);
    assert_eq!(komga(&fake).claim().await.ok(), Some(Owner::Ours));
    assert_eq!(
        asked(&fake),
        [
            (Method::Get, "/api/v1/claim".to_owned()),
            (Method::Post, "/api/v1/claim".to_owned()),
            (Method::Get, "/api/v2/users/me".to_owned()),
            (Method::Get, "/api/v1/libraries".to_owned()),
            (Method::Post, "/api/v1/libraries".to_owned()),
        ]
    );
    assert_eq!(
        headers_of(&fake, Method::Post, "/api/v1/claim"),
        [
            ("X-Komga-Email".to_owned(), EMAIL.to_owned()),
            ("X-Komga-Password".to_owned(), PASSWORD.to_owned()),
        ]
    );
    assert_eq!(
        body_of(&fake, "/api/v1/libraries"),
        Some(serde_json::json!({ "name": "Comics", "root": "/data/media/comics" }))
    );
}

#[tokio::test]
async fn a_comics_library_komga_already_holds_is_not_made_again() {
    let fake = signed_in_and(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, UNCLAIMED)),
        (Method::Post, "/api/v1/claim", Answer::reply(200, ME)),
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(
                200,
                r#"[{"id":"0S31NXXY4VJJX","root":"/data/media/comics/"}]"#,
            ),
        ),
    ]);
    assert_eq!(komga(&fake).claim().await.ok(), Some(Owner::Ours));
    assert!(!fake
        .requests()
        .iter()
        .any(|one| one.method == Method::Post && one.url.ends_with("/libraries")));
}

#[tokio::test]
async fn a_claim_another_caller_took_first_is_answered_as_whoever_holds_it() {
    let fake = Fake::by_route_in_turn(vec![
        (
            Method::Get,
            "/api/v1/claim",
            vec![Answer::reply(200, UNCLAIMED), Answer::reply(200, CLAIMED)],
        ),
        (
            Method::Post,
            "/api/v1/claim",
            vec![Answer::reply(
                400,
                r#"{"message":"This server has already been claimed"}"#,
            )],
        ),
        (
            Method::Get,
            "/api/v2/users/me",
            vec![Answer::reply(401, "")],
        ),
    ]);
    assert_eq!(komga(&fake).claim().await.ok(), Some(Owner::Theirs));
}

#[tokio::test]
async fn a_claim_komga_refuses_while_still_open_is_refused() {
    let fake = Fake::by_route(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, UNCLAIMED)),
        (
            Method::Post,
            "/api/v1/claim",
            Answer::reply(400, "bad email"),
        ),
    ]);
    assert!(komga(&fake).claim().await.is_err());
}

#[tokio::test]
async fn a_claim_whose_administrator_then_does_not_sign_in_is_refused() {
    let fake = Fake::by_route(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, UNCLAIMED)),
        (Method::Post, "/api/v1/claim", Answer::reply(200, ME)),
        (Method::Get, "/api/v2/users/me", Answer::reply(401, "")),
    ]);
    assert!(refused_saying(
        komga(&fake).claim().await,
        "does not sign it in"
    ));
}

#[tokio::test]
async fn a_comics_library_komga_will_not_make_fails_the_claim() {
    let fake = signed_in_and(vec![
        (Method::Get, "/api/v1/claim", Answer::reply(200, UNCLAIMED)),
        (Method::Post, "/api/v1/claim", Answer::reply(200, ME)),
        (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
        (
            Method::Post,
            "/api/v1/libraries",
            Answer::reply(400, r#"{"message":"Library root folder does not exist"}"#),
        ),
    ]);
    assert!(komga(&fake).claim().await.is_err());
}

#[tokio::test]
async fn an_open_komga_is_not_claimed_without_the_minted_administrator() {
    let fake = Fake::by_route(vec![(
        Method::Get,
        "/api/v1/claim",
        Answer::reply(200, UNCLAIMED),
    )]);
    let komga = Komga::new(
        fake.clone(),
        BASE,
        Arc::new(Missing),
        Arc::new(PASSWORD.to_owned()),
    );
    assert!(komga.claim().await.is_err());
    assert_eq!(asked(&fake), [(Method::Get, "/api/v1/claim".to_owned())]);
}
