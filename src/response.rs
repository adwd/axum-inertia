use crate::config::InertiaConfig;
use crate::{page::Page, request::Request};
use axum::response::{Html, IntoResponse, Json};
use http::{HeaderMap, HeaderValue, StatusCode};
use serde::Serialize;

pub(crate) fn escape_page_json(page: String) -> String {
    page.replace('<', "\\u003c")
}

/// An Inertia response.
///
/// More information at:
/// https://inertiajs.com/the-protocol#inertia-responses
pub struct Response<'a> {
    pub(crate) request: Request,
    pub(crate) page: Result<Page<'a>, ()>,
    pub(crate) config: InertiaConfig,
}

impl Response<'_> {
    /// Attaches [flash data] to the page object.
    ///
    /// Flash data is serialized as the top-level `flash` field of the page,
    /// where Inertia v3 clients expose it as `page.flash` and fire a `flash`
    /// event. Unlike props, it is not persisted in the browser history.
    ///
    /// If the value cannot be serialized, the response has a `500 Internal
    /// Server Error` status.
    ///
    /// ```rust
    /// use axum::response::IntoResponse;
    /// use axum_inertia::Inertia;
    /// use serde_json::json;
    ///
    /// async fn handler(i: Inertia) -> impl IntoResponse {
    ///     i.render("Boards/Index", json!({ "boards": [] }))
    ///         .flash(json!({ "success": "Board created" }))
    /// }
    /// ```
    ///
    /// [flash data]: https://inertiajs.com/docs/v3/data-props/flash-data
    pub fn flash<T: Serialize>(mut self, flash: T) -> Self {
        self.page = self.page.and_then(|mut page| {
            page.flash = Some(serde_json::to_value(flash).map_err(|_| ())?);
            Ok(page)
        });
        self
    }
}

impl IntoResponse for Response<'_> {
    fn into_response(self) -> axum::response::Response {
        let page = match self.page {
            Ok(page) => page,
            Err(()) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("Vary", HeaderValue::from_static("X-Inertia"));
        if let Some(version) = &self.config.version() {
            headers.insert("X-Inertia-Version", version.parse().unwrap());
        }
        if self.request.is_xhr {
            headers.insert("X-Inertia", HeaderValue::from_static("true"));
            (headers, Json(page)).into_response()
        } else {
            let page_json = escape_page_json(serde_json::to_string(&page).unwrap());
            let html = (self.config.layout())(page_json);
            (headers, Html(html)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use indoc::formatdoc;

    use super::*;

    #[tokio::test]
    async fn test_into_html_response() {
        let request = Request {
            is_xhr: false,
            ..Request::test_request()
        };
        let page = Page {
            component: "Testing",
            props: serde_json::json!({
                "test": "test",
                "content": "</script><script>alert('xss')</script>",
            }),
            url: "/test".to_string(),
            version: None,
            flash: None,
        };

        let layout = |props| {
            formatdoc! {r#"
            <html>
            <head>
            <title>Foo!</title>
            </head>
            <body>
                <script data-page="app" type="application/json">{}</script>
                <div id="app"></div>
            </body>
            </html>
        "#, props}
            .to_string()
        };

        let config = InertiaConfig::new(Some("123".to_string()), Box::new(layout));

        let response = Response {
            request,
            page: Ok(page),
            config,
        }
        .into_response();

        assert_eq!(response.headers().get("Vary").unwrap(), "X-Inertia");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(body.into()).expect("decoded string");

        assert!(body.contains(r#""test":"test""#));
        assert!(!body.contains("</script><script>alert"));
        assert!(body.contains(r#"\u003c/script>\u003cscript>alert('xss')\u003c/script>"#));
    }

    fn test_page() -> Page<'static> {
        Page {
            component: "Testing",
            props: serde_json::json!({ "test": "test" }),
            url: "/test".to_string(),
            version: None,
            flash: None,
        }
    }

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).expect("json body")
    }

    #[tokio::test]
    async fn it_serializes_flash_data_as_a_top_level_page_field() {
        let config = InertiaConfig::new(None, Box::new(|_| String::new()));
        let response = Response {
            request: Request::test_request(),
            page: Ok(test_page()),
            config,
        }
        .flash(serde_json::json!({ "success": "Saved" }))
        .into_response();

        let page = json_body(response).await;
        assert_eq!(page["flash"], serde_json::json!({ "success": "Saved" }));
        assert!(page["props"].get("flash").is_none());
    }

    #[tokio::test]
    async fn it_omits_flash_when_none_is_set() {
        let config = InertiaConfig::new(None, Box::new(|_| String::new()));
        let response = Response {
            request: Request::test_request(),
            page: Ok(test_page()),
            config,
        }
        .into_response();

        let page = json_body(response).await;
        assert!(page.get("flash").is_none());
    }

    #[test]
    fn it_returns_internal_server_error_when_flash_serialization_fails() {
        struct FailingFlash;
        impl Serialize for FailingFlash {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("serialization failure"))
            }
        }

        let config = InertiaConfig::new(None, Box::new(|_| String::new()));
        let response = Response {
            request: Request::test_request(),
            page: Ok(test_page()),
            config,
        }
        .flash(FailingFlash)
        .into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_json_response_varies_on_x_inertia() {
        let page = Page {
            component: "Testing",
            props: serde_json::json!({ "test": "test" }),
            url: "/test".to_string(),
            version: None,
            flash: None,
        };
        let config = InertiaConfig::new(None, Box::new(|_| String::new()));

        let response = Response {
            request: Request::test_request(),
            page: Ok(page),
            config,
        }
        .into_response();

        assert_eq!(response.headers().get("Vary").unwrap(), "X-Inertia");
    }
}
