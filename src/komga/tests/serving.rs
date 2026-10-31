use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::media::Kind;
use lemonfiber_ports::service::{
    Declaring as _, Encrypting as _, Fronted as _, Library as _, Upkeep as _,
};

use super::{asked, komga, refused_saying, signed_in_and, unsupported, LIBRARIES};

#[tokio::test]
async fn a_rescan_looks_over_every_library_komga_holds() {
    let fake = signed_in_and(vec![
        (Method::Post, "/scan", Answer::reply(202, "")),
        (
            Method::Get,
            "/api/v1/libraries",
            Answer::reply(200, LIBRARIES),
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
    assert!(unsupported(komga.has_item(Kind::Movies, "t").await));
    assert!(unsupported(komga.series_held(1).await));
    assert!(unsupported(komga.known_proxies().await));
    assert!(unsupported(komga.trust_only("10.0.0.1").await));
    assert!(unsupported(komga.allowed_origins().await));
    assert!(unsupported(komga.allow_only("https://example.org").await));
    assert!(unsupported(komga.restart().await));
    assert!(unsupported(komga.declared().await));
    assert!(unsupported(komga.serve_tls("/config/x.p12", "p").await));
    assert!(fake.requests().is_empty());
}
