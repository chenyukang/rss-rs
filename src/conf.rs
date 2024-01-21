#[cfg(test)]
pub(crate) static PAGES_DB: &'static str = "/tmp/pages.db";

#[cfg(not(test))]
pub(crate) static PAGES_DB: &'static str = "./db/pages.db";

pub(crate) static IMAGE_DIR: &'static str = "./pages/images";
pub(crate) static ALL_FEEDS: &'static str = "./db/feeds.md";
