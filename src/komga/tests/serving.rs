use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::media::TitleKind;
use lemonfiber_ports::service::{
    Allowing as _, Declaring as _, Encrypting as _, Fronted as _, Library as _, Recounting as _,
    Tuning as _, Upkeep as _,
};

use super::{asked, komga, refused_saying, signed_in_and, unsupported, FILED, LIBRARIES};

#[tokio::test]
async fn a_rescan_looks_over_every_library_komga_holds_and_makes_none() {
    let fake = signed_in_and(vec![
        (Method::Post, "/scan", Answer::reply(202, "")),
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(200, LIBRARIES),
        ),
        (
            Method::Post,
            "/api/v1/filesystem",
            Answer::reply(200, FILED),
        ),
    ]);
    assert!(komga(&fake).rescan().await.is_ok());
    assert_eq!(
        asked(&fake)
            .into_iter()
            .filter(|(method, _)| *method == Method::Post)
            .collect::<Vec<_>>(),
        [
            (
                Method::Post,
                "/api/v1/libraries/0S31NXXY4VJJX/scan".to_owned()
            ),
            (
                Method::Post,
                "/api/v1/libraries/0S31NXXY4VJJY/scan".to_owned()
            ),
        ]
    );
}

#[tokio::test]
async fn a_rescan_komga_refuses_is_refused() {
    let fake = signed_in_and(vec![
        (Method::Post, "/scan", Answer::reply(500, "")),
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(200, LIBRARIES),
        ),
    ]);
    assert!(komga(&fake).rescan().await.is_err());
}

#[tokio::test]
async fn a_refresh_reads_a_series_afresh_or_else_a_book() {
    let series = signed_in_and(vec![(
        Method::Post,
        "/api/v1/series/0S31NZSGWVZD1/metadata/refresh",
        Answer::reply(202, ""),
    )]);
    assert!(komga(&series).refresh("0S31NZSGWVZD1").await.is_ok());
    let book = signed_in_and(vec![
        (Method::Post, "/api/v1/series/", Answer::reply(404, "")),
        (
            Method::Post,
            "/api/v1/books/0S31NZSH4VVWZ/metadata/refresh",
            Answer::reply(202, ""),
        ),
    ]);
    assert!(komga(&book).refresh("0S31NZSH4VVWZ").await.is_ok());
    assert_eq!(
        asked(&book)
            .into_iter()
            .filter(|(method, _)| *method == Method::Post)
            .count(),
        2
    );
}

#[tokio::test]
async fn a_refresh_of_nothing_komga_holds_is_refused() {
    let fake = signed_in_and(vec![(
        Method::Post,
        "/metadata/refresh",
        Answer::reply(404, ""),
    )]);
    assert!(refused_saying(
        komga(&fake).refresh("0S31NZSGWVZD1").await,
        "no series or book"
    ));
    let failing = signed_in_and(vec![(
        Method::Post,
        "/metadata/refresh",
        Answer::reply(500, ""),
    )]);
    assert!(komga(&failing).refresh("0S31NZSGWVZD1").await.is_err());
    let none = Fake::silent();
    assert!(refused_saying(
        komga(&none).refresh("../x").await,
        "no series or book"
    ));
    assert!(none.requests().is_empty());
}

#[tokio::test]
async fn what_komga_has_no_equivalent_for_is_unsupported_and_asks_nothing() {
    let fake = Fake::silent();
    let komga = komga(&fake);
    assert!(unsupported(komga.series_held(1).await));
    assert!(unsupported(komga.known_proxies().await));
    assert!(unsupported(komga.trust_only("10.0.0.1").await));
    assert!(unsupported(komga.allowed_origins().await));
    assert!(unsupported(komga.allow_only("https://example.org").await));
    assert!(unsupported(komga.restart().await));
    assert!(unsupported(komga.declared().await));
    assert!(unsupported(komga.serve_tls("/config/x.p12", "p").await));
    assert!(unsupported(
        komga
            .allow_origins(&["https://example.org".to_owned()])
            .await
    ));
    assert!(unsupported(
        komga
            .events(
                Some("https://lemonfiber/events"),
                Some("p"),
                Some("k"),
                Some(60)
            )
            .await
    ));
    assert!(unsupported(komga.channels("m").await));
    assert!(unsupported(komga.guide("m", 12).await));
    assert!(unsupported(komga.tune("m", "c").await));
    assert!(unsupported(komga.untune("c").await));
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn a_series_whose_name_or_title_holds_the_term_is_in_the_library() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/series",
        Answer::reply(
            200,
            r#"{"content":[{"id":"0S31NXXY4VJJA","name":"saga-vol","metadata":{"title":"Saga"}},{"id":"0S31NXXY4VJJB","name":"Monstress"}]}"#,
        ),
    )]);
    let komga = komga(&fake);
    assert_eq!(
        komga.has_item(TitleKind::Books, "SAGA").await.ok(),
        Some(true)
    );
    assert_eq!(
        komga.has_item(TitleKind::Comics, "saga").await.ok(),
        Some(true)
    );
    assert_eq!(
        komga.has_item(TitleKind::Comics, "Paper Girls").await.ok(),
        Some(false)
    );
    assert_eq!(
        komga.has_item(TitleKind::Books, "monstr").await.ok(),
        Some(true)
    );
    assert_eq!(
        komga.has_item(TitleKind::Books, "Paper Girls").await.ok(),
        Some(false)
    );
    assert!(fake
        .requests()
        .iter()
        .any(|one| one.url.ends_with("/api/v1/series?search=SAGA&unpaged=true")));
}

#[tokio::test]
async fn nothing_but_comics_and_books_is_in_the_library_and_nothing_is_asked_for_it() {
    let fake = Fake::silent();
    for kind in [
        TitleKind::Tv,
        TitleKind::Movies,
        TitleKind::Music,
        TitleKind::Audiobooks,
        TitleKind::Podcasts,
    ] {
        assert_eq!(komga(&fake).has_item(kind, "t").await.ok(), Some(false));
    }
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn series_it_cannot_read_leave_it_unable_to_say() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/series",
        Answer::reply(200, "{"),
    )]);
    assert!(refused_saying(
        komga(&fake).has_item(TitleKind::Books, "t").await,
        "series Komga holds could not be read"
    ));
}
