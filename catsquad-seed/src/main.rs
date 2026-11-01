use catsquad_client::{Client, ReqwestSender, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_seed::create_img;
use catsquad_shared::{PostState, str_to_uuid};
use clap::{Arg, ArgAction, Command, arg};
use http::header;
use std::fmt::Debug;
use tokio::{spawn, task::spawn_local};
use tokio_util::task::TaskTracker;

const ORIGIN: &'static str = "http://localhost:3000";
const PASSWORD: &'static str = "A5%prime1@heyadora.com";

#[tokio::main(flavor = "local")]
async fn main() {
    init_log();

    let sender = ReqwestSender::new(ORIGIN);
    let client = Client::new(sender);

    let command = Command::new("seed")
        .about("data seeder")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("user").subcommand_required(true).subcommand(
                Command::new("add").subcommand(
                    Command::new("batch")
                        .about("create users ")
                        .arg(
                            Arg::new("number")
                                .short('n')
                                .long("number")
                                .help("number of users it will create offset..number")
                                .action(ArgAction::Set)
                                .num_args(1..)
                                .value_parser(clap::value_parser!(usize))
                                .required(true),
                        )
                        .arg(
                            Arg::new("offset")
                                .long("offset")
                                .help("number to skip when creating users offset..number")
                                .action(ArgAction::Set)
                                .num_args(1..)
                                .value_parser(clap::value_parser!(usize))
                                .required(false),
                        )
                        .arg_required_else_help(true),
                ),
            ),
        )
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
        Some(("user", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches)) => match sub_matches.subcommand() {
                Some(("batch", sub_matches)) => {
                    let number = *sub_matches.get_one::<usize>("number").unwrap();
                    let offset = *sub_matches.get_one::<usize>("offset").unwrap_or(&0);
                    command_user_add_batch(client.clone(), offset, number).await;
                }
                _ => {
                    info!("not implemented yet");
                }
            },
            _ => unreachable!(),
        },
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

async fn command_user_add_batch<TSender>(client: Client<TSender>, offset: usize, n: usize)
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    let tracker = TaskTracker::new();
    for i in offset..n {
        let client = client.clone();
        tracker.spawn_local(async move {
            let username = format!("prime{i}");
            let email = format!("prime{i}@heyadora.com");
            let password = PASSWORD;

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
                    if v.to_email != email {
                        return None;
                    }

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

            let token = str_to_uuid(token);
            let _res = client
                .user_add(username, token, password)
                .send()
                .await
                .into_json()
                .await
                .unwrap();

            info!("user registered {email}");
        });
    }
    tracker.close();
    tracker.wait().await;
}

#[cfg(test)]
#[tokio::test(flavor = "local")]
async fn test_command_user_add() {
    use catsquad_api::TestServer;

    init_log();

    let server = TestServer::new(0, "test_command_user_add").await;

    // assert success
    {
        server
            .user_add_full("prime0", "prime0@heyadora.com", PASSWORD)
            .await;
        command_user_add_batch(server.client.clone(), 1, 10).await;
    }
}

// .header_add(
//     header::COOKIE,
//     create_auth_cookie_str(uuid_to_str(session_token)),
// )
// let res = client.user_add(username, token, password).send().await;
// let headers = res.get_headers().unwrap();
// let res = res.into_json().await.unwrap();
// let session_token = auth_token_get(&headers, header::SET_COOKIE).unwrap();
