use askama::Template;
use crate::media::MediaFile;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub files: Vec<MediaFile>,
}
