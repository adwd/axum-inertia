use std::sync::Arc;

type LayoutResolver = Box<dyn Fn(String) -> String + Send + Sync>;

struct Inner {
    version: Option<String>,
    layout: LayoutResolver,
}

#[derive(Clone)]
pub struct InertiaConfig {
    inner: Arc<Inner>,
}

impl InertiaConfig {
    /// Constructs a new InertiaConfig object.
    ///
    /// `layout` receives the JSON-encoded page object and renders the
    /// initial page load. The page JSON belongs in a
    /// `<script data-page="app" type="application/json">` element,
    /// with a separate `<div id="app"></div>` mount point. See the
    /// [crate::vite] module for a Vite implementation.
    pub fn new(version: Option<String>, layout: LayoutResolver) -> InertiaConfig {
        let inner = Inner { version, layout };
        InertiaConfig {
            inner: Arc::new(inner),
        }
    }

    /// Returns a cloned optional version string.
    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }

    /// Returns a reference to the layout function.
    pub fn layout(&self) -> &LayoutResolver {
        &self.inner.layout
    }
}
