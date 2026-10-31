use ab_glyph::{FontRef, PxScale};
use catsquad_client::{Client, ReqwestSender, Response, Sender};
// use artbounty::{
//     api::{Api, ApiNative, ServerReqImg},
//     path::{
//         PATH_API_LOGIN, PATH_API_POST_ADD, PATH_API_REGISTER, PATH_API_SEND_EMAIL_INVITE,
//         PATH_API_USER,
//     },
// };
use catsquad_log::prelude::*;
use catsquad_seed::create_img;
use catsquad_shared::{PostState, str_to_uuid};
use clap::{Command, arg};
use http::header;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text;
use rand::Rng;
use std::{collections::HashMap, env, fmt::Debug, time::Duration};
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
        Some(("register", sub_matches)) => {
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
    create_img(path, img_text);

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
        .post_update_image_add(post1.id, vec![path])
        .header_add(header::COOKIE, format!("authorization=Bearer {token}"))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let post1 = client
        .post_update_state(post1.id, PostState::Active)
        .header_add(header::COOKIE, format!("authorization=Bearer {token}"))
        .send()
        .await
        .into_json()
        .await
        .unwrap();
}

async fn command_user_add<TSender>(client: Client<TSender>, offset: usize, n: usize)
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    for i in offset..n {
        let username = format!("prime{i}");
        let email = format!("prime{i}@heyadora.com");
        let password = "A5%prime1@heyadora.com";

        let _invite = client
            .invite_add(email.clone())
            .send()
            .await
            .into_json()
            .await
            .unwrap();

        let token = client
            .test_backdoor_email_sent_get_all()
            .send()
            .await
            .into_json()
            .await
            .inspect(|v| trace!("EMAILS RECEIVED {v:#?}"))
            .unwrap()
            .into_iter()
            .find_map(|v| {
                v.body
                    .split_whitespace()
                    .into_iter()
                    .find(|v| v.contains("http://") && v.contains("register"))
                    .map(|v| v.to_string())
            })
            .unwrap()
            .split("token=")
            .nth(1)
            .unwrap()
            .to_string();

        trace!("TOKEN FOUND {token}");

        let token = str_to_uuid(token);

        // .header_add(
        //     header::COOKIE,
        //     create_auth_cookie_str(uuid_to_str(session_token)),
        // )
        // server
        //     .inject_header(
        //         header::COOKIE,
        //         create_auth_cookie_str(uuid_to_str(session1)),
        //     )
        //     .await;

        // let invite = self.invite_add(email.clone()).await.unwrap();
        // let invite = self
        //     .state
        //     .db
        //     .invite_get_all()
        //     .await
        //     .unwrap()
        //     .into_iter()
        //     .find(|v| !v.used && v.email == email)
        //     .unwrap()
        //     .token
        //     .clone();
        let res = client.user_add(username, token, password).send().await;
        let headers = res.get_headers().unwrap();
        let res = res.into_json().await.unwrap();

        // use catsquad_api::auth::auth_token_get;
        // let session_token = auth_token_get(&headers, header::SET_COOKIE).unwrap();

        // trace!("SESSION FOUND {session_token}");

        // let session_token = str_to_uuid(session_token);

        // let headers = res.get_headers().unwrap();
        // let session_token = auth_token_get(&headers, header::SET_COOKIE).unwrap();
        // let session_token = str_to_uuid(session_token);
        // let _res = res.into_json().await.unwrap();

        // // let session_key = res.get_auth_token().unwrap();
        // // let res = res.into_res().await;
        // let user = self.state.db.user_get_by_email(email).await.unwrap();
        // (user, session_token)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_command_user_add() {
    use catsquad_api::{TestServer, utils::get_time_micro};

    init_log();

    let time = get_time_micro() as u64;
    let server = TestServer::new(time, "test_command_user_add").await;

    // assert success
    {
        let (user, user_token) = server
            .user_add_full("prime1", "prime1@heyadora.com", "passwordA1@39jes0")
            .await;
        command_user_add(server.client.clone(), 0, 1).await;
        // let result = server.user_add("hey", 0_u128.to_be_bytes(), "").await;
        // assert!(
        //     matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_none() && password.is_some())
        // );
    }
}
