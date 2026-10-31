use lemonfiber_fixtures::http::Answer;
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{HowFar, Image, Item, ItemDetail, Medium, Picture, Screening as _};

use super::super::screening::a_date;
use super::{asked, komga, signed_in_and, unsupported};
use lemonfiber_fixtures::http::Fake;

const SERIES_PAGE: &str = r#"{"content":[
  {"id":"0S31NZSGWVZD1","name":"Probe Series","booksCount":2,
   "metadata":{"title":"The Probe","summary":"  ","genres":["comedy"]},
   "booksMetadata":{"releaseDate":"2001-02-03","summary":"From the books."}},
  {"id":"bad/id","name":"Not one","booksCount":1},
  {"id":"0S31NZSGWVZD2","name":"Second","booksCount":0,
   "metadata":{"title":""},"booksMetadata":{"releaseDate":"someday"}},
  {"id":"0S31NZSGWVZD3","name":"Third","booksCount":1}
]}"#;

const ONE_SERIES: &str = r#"{"id":"0S31NZSGWVZD1","name":"Probe Series","booksCount":2,
  "metadata":{"title":"The Probe","summary":"A probe.","genres":["comedy","drama"]},
  "booksMetadata":{"releaseDate":"2001-02-03","summary":"From the books."}}"#;

const ONE_BOOK: &str = r#"{"id":"0S31NZSH4VVWZ","name":"Probe Series 001",
  "metadata":{"title":"","summary":"The first.","releaseDate":"1999-12-31"}}"#;

fn item(id: &str, title: &str, year: Option<u16>, poster: bool) -> Item {
    Item {
        id: id.to_owned(),
        title: title.to_owned(),
        year,
        medium: Medium::Other,
        holds: lemonfiber_ports::service::Holds {
            poster,
            backdrop: false,
            plays: false,
        },
    }
}

#[tokio::test]
async fn holdings_are_komga_s_series_by_title_as_many_as_asked() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/series?",
        Answer::reply(200, SERIES_PAGE),
    )]);
    let held = komga(&fake).holdings(None, 2).await;
    assert_eq!(
        held.ok(),
        Some(vec![
            item("0S31NZSGWVZD1", "The Probe", Some(2001), true),
            item("0S31NZSGWVZD2", "Second", None, false),
        ])
    );
    assert_eq!(
        asked(&fake).last(),
        Some(&(
            Method::Get,
            "/api/v1/series?size=2&sort=metadata.titleSort%2Casc".to_owned()
        ))
    );
}

#[tokio::test]
async fn asking_for_no_holdings_asks_komga_nothing() {
    let fake = Fake::silent();
    assert_eq!(komga(&fake).holdings(None, 0).await.ok(), Some(Vec::new()));
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn a_series_title_says_what_its_metadata_and_its_books_say() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/series/0S31NZSGWVZD1",
        Answer::reply(200, ONE_SERIES),
    )]);
    assert_eq!(
        komga(&fake).title(None, "0S31NZSGWVZD1").await.ok(),
        Some(Some(ItemDetail {
            item: item("0S31NZSGWVZD1", "The Probe", Some(2001), true),
            overview: Some("A probe.".to_owned()),
            minutes: None,
            genres: vec!["comedy".to_owned(), "drama".to_owned()],
            certificate: None,
            released: Some("2001-02-03".to_owned()),
            seasons: Vec::new(),
        }))
    );
}

#[tokio::test]
async fn a_series_saying_nothing_of_itself_is_described_by_its_books() {
    let fake = signed_in_and(vec![(
        Method::Get,
        "/api/v1/series/0S31NZSGWVZD1",
        Answer::reply(
            200,
            r#"{"id":"0S31NZSGWVZD1","name":"Probe","booksMetadata":{"summary":"From the books."}}"#,
        ),
    )]);
    let detail = komga(&fake)
        .title(None, "0S31NZSGWVZD1")
        .await
        .ok()
        .flatten();
    assert_eq!(
        detail.and_then(|detail| detail.overview),
        Some("From the books.".to_owned())
    );
}

#[tokio::test]
async fn an_id_no_series_holds_is_read_as_a_book() {
    let fake = signed_in_and(vec![
        (
            Method::Get,
            "/api/v1/series/0S31NZSH4VVWZ",
            Answer::reply(404, ""),
        ),
        (
            Method::Get,
            "/api/v1/books/0S31NZSH4VVWZ",
            Answer::reply(200, ONE_BOOK),
        ),
    ]);
    assert_eq!(
        komga(&fake).title(None, "0S31NZSH4VVWZ").await.ok(),
        Some(Some(ItemDetail {
            item: item("0S31NZSH4VVWZ", "Probe Series 001", Some(1999), true),
            overview: Some("The first.".to_owned()),
            minutes: None,
            genres: Vec::new(),
            certificate: None,
            released: Some("1999-12-31".to_owned()),
            seasons: Vec::new(),
        }))
    );
}

#[tokio::test]
async fn an_id_komga_holds_nothing_under_is_no_title() {
    let fake = signed_in_and(vec![
        (Method::Get, "/api/v1/series/", Answer::reply(404, "")),
        (Method::Get, "/api/v1/books/", Answer::reply(404, "")),
    ]);
    assert_eq!(
        komga(&fake).title(None, "0S31NZSH4VVWZ").await.ok(),
        Some(None)
    );
}

#[tokio::test]
async fn an_id_not_spelled_as_komga_s_is_no_title_and_no_picture_and_nothing_is_asked() {
    let fake = Fake::silent();
    let komga = komga(&fake);
    assert_eq!(komga.title(None, "../x").await.ok(), Some(None));
    assert_eq!(
        komga.picture(None, "../x", Picture::Poster).await.ok(),
        Some(None)
    );
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn a_poster_is_the_series_thumbnail_or_else_the_book_s() {
    let series = signed_in_and(vec![(
        Method::Get,
        "/api/v1/series/0S31NZSGWVZD1/thumbnail",
        Answer::served(200, "image/jpeg", "jpeg-bytes"),
    )]);
    assert_eq!(
        komga(&series)
            .picture(None, "0S31NZSGWVZD1", Picture::Poster)
            .await
            .ok(),
        Some(Some(Image {
            media_type: "image/jpeg".to_owned(),
            bytes: b"jpeg-bytes".to_vec(),
        }))
    );
    let book = signed_in_and(vec![
        (
            Method::Get,
            "/api/v1/series/0S31NZSH4VVWZ/thumbnail",
            Answer::reply(404, ""),
        ),
        (
            Method::Get,
            "/api/v1/books/0S31NZSH4VVWZ/thumbnail",
            Answer::served(200, "image/png", "png-bytes"),
        ),
    ]);
    assert_eq!(
        komga(&book)
            .picture(None, "0S31NZSH4VVWZ", Picture::Poster)
            .await
            .ok()
            .flatten()
            .map(|image| image.media_type),
        Some("image/png".to_owned())
    );
}

#[tokio::test]
async fn a_thumbnail_that_is_not_a_picture_is_none_and_a_refusal_is_refused() {
    let page = signed_in_and(vec![(
        Method::Get,
        "/thumbnail",
        Answer::served(200, "text/html", "<html>"),
    )]);
    assert_eq!(
        komga(&page)
            .picture(None, "0S31NZSGWVZD1", Picture::Poster)
            .await
            .ok(),
        Some(None)
    );
    let refusing = signed_in_and(vec![(Method::Get, "/thumbnail", Answer::reply(500, ""))]);
    assert!(komga(&refusing)
        .picture(None, "0S31NZSGWVZD1", Picture::Poster)
        .await
        .is_err());
    let unreachable = signed_in_and(vec![(Method::Get, "/thumbnail", Answer::Silent)]);
    assert!(komga(&unreachable)
        .picture(None, "0S31NZSGWVZD1", Picture::Poster)
        .await
        .is_err());
}

#[tokio::test]
async fn a_thumbnail_refused_for_its_session_is_asked_again_once_signed_in_again() {
    let fake = Fake::by_route_in_turn(vec![
        (
            Method::Get,
            "/thumbnail",
            vec![
                Answer::reply(401, ""),
                Answer::served(200, "image/jpeg", "jpeg-bytes"),
            ],
        ),
        (Method::Get, "/api/v2/users/me", vec![super::signed_in()]),
    ]);
    assert!(komga(&fake)
        .picture(None, "0S31NZSGWVZD1", Picture::Poster)
        .await
        .ok()
        .flatten()
        .is_some());
    assert_eq!(fake.requests().len(), 4);
}

#[tokio::test]
async fn there_is_no_backdrop_and_nothing_is_asked_for_one() {
    let fake = Fake::silent();
    assert_eq!(
        komga(&fake)
            .picture(None, "0S31NZSGWVZD1", Picture::Backdrop)
            .await
            .ok(),
        Some(None)
    );
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn whatever_names_a_member_or_a_device_and_what_is_playing_is_unsupported() {
    let fake = Fake::silent();
    let komga = komga(&fake);
    assert!(unsupported(komga.signed_in("m", "d").await));
    assert!(unsupported(komga.title(Some("m"), "0S31NZSGWVZD1").await));
    assert!(unsupported(komga.part_way("m", 1).await));
    assert!(unsupported(
        komga
            .progressed(
                "m",
                "0S31NZSGWVZD1",
                &HowFar {
                    position: 1,
                    ended: false
                }
            )
            .await
    ));
    assert!(unsupported(komga.sign_out("d").await));
    assert!(unsupported(
        komga
            .picture(Some("m"), "0S31NZSGWVZD1", Picture::Poster)
            .await
    ));
    assert!(unsupported(komga.holdings(Some("m"), 1).await));
    assert!(unsupported(komga.playing(None).await));
    assert!(fake.requests().is_empty());
}

#[test]
fn a_date_is_written_as_year_month_day() {
    assert!(a_date("2001-02-03"));
    for not in ["2001-2-03", "20010203xx", "2001/02/03", "someday", ""] {
        assert!(!a_date(not), "{not}");
    }
}
