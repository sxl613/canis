use crate::ListParams;
use crate::media::MediaFile;
use crate::media::PaginatedMedia;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub paginated: PaginatedMedia,
    pub query: ListParams,
}

#[derive(Template)]
#[template(path = "watch.html")]
pub struct WatchTemplate {
    pub video: MediaFile,
}
