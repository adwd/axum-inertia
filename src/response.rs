use crate::config::InertiaConfig;
use crate::{page::Page, request::Request};
use axum::response::{Html, IntoResponse, Json};
use http::{HeaderMap, HeaderValue, StatusCode};

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

    #[test]
    fn test_into_json_response_varies_on_x_inertia() {
        let page = Page {
            component: "Testing",
            props: serde_json::json!({ "test": "test" }),
            url: "/test".to_string(),
            version: None,
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
