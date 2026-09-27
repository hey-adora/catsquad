use super::author_state::{AuthorStage, AuthorState};
use crate::page::{create_client, index::api_gallery::GalleryApi};
use leptos::prelude::*;

#[component]
pub fn ProfileAuthor(gallery_api: GalleryApi) -> impl IntoView {
    let author_state = AuthorState::new(create_client());
    view! {}
}
