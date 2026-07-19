use serde::Serialize;
use serde_json::Value;

/// Holds data for the Inertia page object.
///
/// Serializes to JSON. Included in the `script[data-page]` element of
/// the initial HTML page, or sent as the payload for Inertia requests.
///
/// More info at: https://inertiajs.com/the-protocol#the-page-object
#[derive(Serialize)]
pub(crate) struct Page<'a> {
    pub(crate) component: &'a str,
    pub(crate) props: Value,
    pub(crate) url: String,
    pub(crate) version: Option<String>,
}
