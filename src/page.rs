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
    /// Inertia v3 flash data: one-time values (e.g. toast messages) that
    /// are exposed as `page.flash` and are not persisted in history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) flash: Option<Value>,
}
