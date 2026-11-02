use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Failure, FiledAt, Furnished, Furnishing as _};

use super::{asked, body_of, komga, signed_in_and, FILED};

const COMICS: &str = "/data/media/comics";

const BOOKS: &str = "/data/media/books";

const BOTH_FILED: &str = r#"{"parent":"/data/media","directories":[{"type":"directory","name":"books","path":"/data/media/books"},{"type":"directory","name":"comics","path":"/data/media/comics"}],"files":[]}"#;

fn filed(media_type: &str, folder: &str) -> FiledAt {
    FiledAt {
        media_type: media_type.to_owned(),
        folder: folder.to_owned(),
    }
}

fn answered(media_type: &str, held: bool) -> Furnished {
    Furnished {
        media_type: media_type.to_owned(),
        held,
    }
}

fn posted(fake: &Fake) -> Vec<String> {
    asked(fake)
        .into_iter()
        .filter(|(method, _)| *method != Method::Get)
        .map(|(_, path)| path)
        .collect()
}

#[tokio::test]
async fn a_library_is_added_for_comics_and_books_whose_folders_exist_and_none_reads() {
    let fake = signed_in_and(vec![
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(
                200,
                r#"[{"id":"0S31NXXY4VJJX","name":"Manga","root":"/data/media/manga"}]"#,
            ),
        ),
        (
            Method::Post,
            "/api/v1/filesystem",
            Answer::reply(200, BOTH_FILED),
        ),
        (Method::Post, "/api/v1/libraries", Answer::reply(200, "{}")),
    ]);
    let furnished = komga(&fake)
        .furnish(&[filed("books", BOOKS), filed("comics", COMICS)])
        .await;
    assert_eq!(
        furnished.ok(),
        Some(vec![answered("books", true), answered("comics", true)])
    );
    assert_eq!(
        posted(&fake),
        [
            "/api/v1/filesystem",
            "/api/v1/libraries",
            "/api/v1/filesystem",
            "/api/v1/libraries",
        ]
    );
    let bodies: Vec<serde_json::Value> = fake
        .requests()
        .into_iter()
        .filter(|one| one.method == Method::Post)
        .filter_map(|one| one.body)
        .filter_map(|body| serde_json::from_str(&body).ok())
        .collect();
    assert_eq!(
        bodies,
        [
            serde_json::json!({ "path": "/data/media", "showFiles": false }),
            serde_json::json!({ "name": "Books", "root": BOOKS }),
            serde_json::json!({ "path": "/data/media", "showFiles": false }),
            serde_json::json!({ "name": "Comics", "root": COMICS }),
        ]
    );
}

#[tokio::test]
async fn a_library_already_reading_the_folder_is_left_alone_and_answered_held() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/libraries",
        Answer::reply(
            200,
            r#"[{"id":"0S31NXXY4VJJX","name":"My comics","root":"/data/media/comics/"}]"#,
        ),
    )]);
    let furnished = komga(&fake).furnish(&[filed("comics", COMICS)]).await;
    assert_eq!(furnished.ok(), Some(vec![answered("comics", true)]));
    assert!(posted(&fake).is_empty());
}

#[tokio::test]
async fn a_library_of_that_name_reading_another_folder_is_neither_re_pointed_nor_doubled() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/libraries",
        Answer::reply(
            200,
            r#"[{"id":"0S31NXXY4VJJX","name":"comics","root":"/srv/comics"}]"#,
        ),
    )]);
    let furnished = komga(&fake).furnish(&[filed("comics", COMICS)]).await;
    assert_eq!(furnished.ok(), Some(vec![answered("comics", false)]));
    assert!(posted(&fake).is_empty());
}

#[tokio::test]
async fn a_media_type_komga_does_not_take_is_not_held_and_asks_nothing() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/libraries",
        Answer::reply(200, "[]"),
    )]);
    let furnished = komga(&fake)
        .furnish(&[
            filed("movies", "/data/media/movies"),
            filed("tv", "/data/media/tv"),
            filed("music", "/data/media/music"),
            filed("audiobooks", "/data/media/audiobooks"),
            filed("podcasts", "/data/media/podcasts"),
        ])
        .await;
    assert_eq!(
        furnished.ok(),
        Some(
            ["movies", "tv", "music", "audiobooks", "podcasts"]
                .map(|media_type| answered(media_type, false))
                .to_vec()
        )
    );
    assert!(posted(&fake).is_empty());
}

#[tokio::test]
async fn a_folder_that_does_not_exist_is_given_no_library() {
    for listing in [
        Answer::reply(
            200,
            r#"{"parent":"/data/media","directories":[],"files":[]}"#,
        ),
        Answer::reply(
            200,
            r#"{"parent":"/data/media","directories":[{"type":"directory","name":"comics","path":"/elsewhere/comics"}],"files":[]}"#,
        ),
        Answer::reply(404, ""),
        Answer::reply(400, r#"{"message":"Path does not exist"}"#),
    ] {
        let fake = signed_in_and(vec![
            (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
            (Method::Post, "/api/v1/filesystem", listing),
        ]);
        let furnished = komga(&fake).furnish(&[filed("comics", COMICS)]).await;
        assert_eq!(furnished.ok(), Some(vec![answered("comics", false)]));
        assert_eq!(posted(&fake), ["/api/v1/filesystem"]);
    }
}

#[tokio::test]
async fn a_folder_with_nothing_above_it_is_given_no_library() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/libraries",
        Answer::reply(200, "[]"),
    )]);
    let furnished = komga(&fake)
        .furnish(&[filed("comics", "/"), filed("books", "")])
        .await;
    assert_eq!(
        furnished.ok(),
        Some(vec![answered("comics", false), answered("books", false)])
    );
    assert!(posted(&fake).is_empty());
}

#[tokio::test]
async fn a_library_komga_refuses_to_add_is_answered_not_held() {
    let fake = signed_in_and(vec![
        (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
        (
            Method::Post,
            "/api/v1/filesystem",
            Answer::reply(200, FILED),
        ),
        (
            Method::Post,
            "/api/v1/libraries",
            Answer::reply(400, r#"{"message":"Library root folder overlaps another"}"#),
        ),
    ]);
    let furnished = komga(&fake).furnish(&[filed("comics", COMICS)]).await;
    assert_eq!(furnished.ok(), Some(vec![answered("comics", false)]));
    assert_eq!(
        body_of(&fake, "/api/v1/libraries"),
        Some(serde_json::json!({ "name": "Comics", "root": COMICS }))
    );
}

#[tokio::test]
async fn a_komga_failing_or_unreadable_or_unreachable_fails_the_furnishing() {
    let failing = [
        signed_in_and(vec![(
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(500, ""),
        )]),
        signed_in_and(vec![(
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(200, "<html>"),
        )]),
        signed_in_and(vec![
            (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
            (Method::Post, "/api/v1/filesystem", Answer::reply(500, "")),
        ]),
        signed_in_and(vec![
            (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
            (
                Method::Post,
                "/api/v1/filesystem",
                Answer::reply(200, FILED),
            ),
            (Method::Post, "/api/v1/libraries", Answer::reply(500, "")),
        ]),
        signed_in_and(vec![
            (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
            (Method::Post, "/api/v1/filesystem", Answer::reply(401, "")),
        ]),
        signed_in_and(vec![
            (Method::Get, "/api/v1/libraries", Answer::reply(200, "[]")),
            (
                Method::Post,
                "/api/v1/filesystem",
                Answer::reply(200, FILED),
            ),
            (Method::Post, "/api/v1/libraries", Answer::reply(401, "")),
        ]),
    ];
    for fake in failing {
        assert!(komga(&fake)
            .furnish(&[filed("comics", COMICS)])
            .await
            .is_err());
    }
    let silent = Fake::silent();
    assert!(matches!(
        komga(&silent).furnish(&[filed("comics", COMICS)]).await,
        Err(Failure::Unavailable { .. })
    ));
}

#[tokio::test]
async fn nothing_is_renamed_re_pointed_or_removed() {
    let fake = signed_in_and(vec![
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(
                200,
                r#"[{"id":"0S31NXXY4VJJX","name":"Comics","root":"/srv/old-comics"},{"id":"0S31NXXY4VJJY","name":"Shelf","root":"/data/media/books"}]"#,
            ),
        ),
        (Method::Post, "/api/v1/libraries", Answer::reply(200, "{}")),
    ]);
    let furnished = komga(&fake)
        .furnish(&[filed("books", BOOKS), filed("comics", COMICS)])
        .await;
    assert_eq!(
        furnished.ok(),
        Some(vec![answered("books", true), answered("comics", false)])
    );
    assert!(fake.requests().iter().all(|one| one.method == Method::Get));
}
