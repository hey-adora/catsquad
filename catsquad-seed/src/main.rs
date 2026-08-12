use ab_glyph::{FontRef, PxScale};
use catsquad_client::{Client, ReqwestSender};
// use artbounty::{
//     api::{Api, ApiNative, ServerReqImg},
//     path::{
//         PATH_API_LOGIN, PATH_API_POST_ADD, PATH_API_REGISTER, PATH_API_SEND_EMAIL_INVITE,
//         PATH_API_USER,
//     },
// };
use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use clap::{Command, arg};
use http::header;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text;
use rand::Rng;
use std::{collections::HashMap, env, time::Duration};
use tokio::fs;
// use tracing::{info, trace};

const ORIGIN: &'static str = "http://localhost:3000";

#[tokio::main]
async fn main() {
    init_log();

    let command = Command::new("seed")
        .about("data seeder")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("post")
                .about("create posts")
                .arg(arg!(--"token" <TOKEN>).required(true))
                .arg(arg!(--"count" <COUNT>).required(true))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("tag_example")
                .about("create tag_example posts")
                .arg(arg!(--"token" <TOKEN>).required(true))
                .arg_required_else_help(true),
        );
    let matches = command.get_matches();
    match matches.subcommand() {
        Some(("post", sub_matches)) => {
            let token = sub_matches.get_one::<String>("token").unwrap();
            let count = sub_matches.get_one::<String>("count").unwrap();
            let count = u32::from_str_radix(count, 10).unwrap();

            for i in 0..count {
                post_add(
                    token,
                    i.to_string(),
                    format!("title{i}"),
                    format!("description{i}"),
                    format!("tag{i}"),
                )
                .await;
            }
        }
        Some(("tag_example", sub_matches)) => {
            let token = sub_matches.get_one::<String>("token").unwrap();

            post_add(token, "one", format!("one"), format!(""), format!("one")).await;

            post_add(
                token,
                "two",
                format!("two"),
                format!(""),
                format!("one two"),
            )
            .await;

            post_add(
                token,
                "three",
                format!("three"),
                format!(""),
                format!("one two three"),
            )
            .await;
        }
        _ => unreachable!(),
    }
}

pub async fn post_add(
    token: impl Into<String>,
    img_text: impl Into<String>,
    title: impl Into<String>,
    description: impl Into<String>,
    tags: impl Into<String>,
) {
    let img_text = img_text.into();
    let token = token.into();
    let title = title.into();
    let description = description.into();
    let tags = tags.into();
    let path = "/tmp/img.png";
    let mut rng = rand::rng();
    let mut image = RgbImage::new(200, 200);
    let r = rng.random_range(200u8..255);
    let g = rng.random_range(200u8..255);
    let b = rng.random_range(200u8..255);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = image::Rgb([r, g, b]);
    }
    let height = 64.0;
    let scale = PxScale {
        x: height * 2.0,
        y: height,
    };

    let font = FontRef::try_from_slice(include_bytes!("../../assets/noto_sans.ttf")).unwrap();
    let img = draw_text(
        &mut image,
        Rgb([0u8, 0u8, 0u8]),
        0,
        50,
        scale,
        &font,
        &img_text,
    );
    img.save(path).unwrap();

    let img = fs::read(path).await.unwrap();

    let sender = ReqwestSender::new(ORIGIN);
    let client = Client::new(sender);
    let post1 = client
        .post_add(title, description, tags)
        .header_add(header::COOKIE, format!("authorization=Bearer {token}"))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let files = client
        .post_update_file_add(post1.key.clone(), vec![path])
        .header_add(header::COOKIE, format!("authorization=Bearer {token}"))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let post1 = client
        .post_update_state(post1.key.clone(), PostState::Active)
        .header_add(header::COOKIE, format!("authorization=Bearer {token}"))
        .send()
        .await
        .into_json()
        .await
        .unwrap();
}
