use catsquad_log::prelude::*;
use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::{hook::Spawner, page::create_client};

#[derive(Clone, Copy)]
pub struct PostLike {
    pub stage: StoredValue<Box<dyn Fn() -> PostLikeStage + Sync + Send + 'static>>,
    pub on_like: StoredValue<Box<dyn Fn() + Sync + Send + 'static>>,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum PostLikeStage {
    #[default]
    Loading,
    Liked,
    Unliked,
}

pub fn use_post_like(post_key: Memo<Option<String>>) -> PostLike {
    // let api = ApiWeb::new();
    let spawner = Spawner::new();
    let stage = RwSignal::new(PostLikeStage::Loading);
    let stage_view = move || {
        if spawner.is_busy.get() {
            PostLikeStage::Loading
        } else {
            stage.get()
        }
    };

    Effect::new({
        let post_key = post_key.clone();
        move || {
            let Some(post_key) = post_key.get() else {
                return;
            };
            let client = create_client();
            spawner.spawn(async move {
                let result = client
                    .post_like_get_by_post(post_key)
                    .send()
                    .await
                    .into_res()
                    .await;
                match result {
                    Ok(liked) => {
                        if liked {
                            stage.set(PostLikeStage::Liked);
                        } else {
                            stage.set(PostLikeStage::Unliked);
                        }
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                        stage.set(PostLikeStage::Unliked);
                    }
                };
            });
        }
    });

    let on_like = move || {
        let Some(post_id) = post_key.get() else {
            return;
        };
        match stage.get_untracked() {
            PostLikeStage::Loading => {
                //
            }
            PostLikeStage::Liked => {
                spawner.spawn(async move {
                    let client = create_client();
                    let result = client
                        .post_like_remove(post_id)
                        .send()
                        .await
                        .into_res()
                        .await;
                    match result {
                        Ok(_result) => {
                            stage.set(PostLikeStage::Unliked);
                        }
                        Err(err) => {
                            error!("use_post_like: {err}");
                            stage.set(PostLikeStage::Unliked);
                        }
                    };
                });
            }
            PostLikeStage::Unliked => {
                spawner.spawn(async move {
                    let client = create_client();
                    let result = client.post_like_add(post_id).send().await.into_res().await;
                    match result {
                        Ok(_result) => {
                            stage.set(PostLikeStage::Liked);
                        }
                        Err(err) => {
                            error!("use_post_like: {err}");
                            stage.set(PostLikeStage::Unliked);
                        }
                    };
                });
            }
        }
    };

    PostLike {
        stage: StoredValue::new(Box::new(stage_view)),
        on_like: StoredValue::new(Box::new(on_like)),
    }
}
