use askama::Template;
use crate::media::MediaFile;
use crate::ListParams;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub files: Vec<MediaFile>,
    pub query: ListParams,
}


#[derive(Template)]
#[template(path = "watch.html")]
pub struct WatchTemplate {
    pub video: MediaFile,
}
