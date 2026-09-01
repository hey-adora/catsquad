use catsquad_api::TestServer;
use catsquad_db::id_to_string;
use catsquad_log::prelude::*;
use std::env;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    init_log();

    let args: Vec<String> = env::args().collect();
    let sample_count = args
        .get(1)
        .and_then(|v| usize::from_str_radix(v, 10).ok())
        .unwrap_or(100);

    info!("sample count set: {sample_count}");
    bench_user_add(sample_count).await;
    bench_invite_add(sample_count).await;
}

async fn bench_user_add(n: usize) {
    let server = crate::TestServer::new().await;
    let mut usernames = Vec::new();
    for i in 0..n {
        let username = format!("prime{i}");
        let email = format!("prime{i}@heyadora.com");
        server.invite_add(&email).await.unwrap();
        usernames.push(username);
    }
    let keys = server
        .state
        .db
        .invite_get_all()
        .await
        .unwrap()
        .into_iter()
        .map(|v| id_to_string(v.id))
        .collect::<Vec<String>>();

    let itr = keys.into_iter().zip(usernames);
    let time = Instant::now();
    for (key, username) in itr {
        server
            .user_add(username, key, "A5%prime@heyadora.com")
            .await
            .unwrap();
    }
    let elapsed = time.elapsed() / n as u32;
    info!("user_add {elapsed:?}");
}

async fn bench_invite_add(n: usize) {
    let server = crate::TestServer::new().await;

    let mut emails = Vec::new();
    for i in 0..n {
        let email = format!("prime{i}@heyadora.com");
        emails.push(email);
    }

    let itr = emails.into_iter();
    let time = Instant::now();
    for email in itr {
        server.invite_add(email).await.unwrap();
    }
    let elapsed = time.elapsed() / n as u32;
    info!("invite_add {elapsed:?}");
}
