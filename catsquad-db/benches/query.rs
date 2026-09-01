use catsquad_db::{Db, Uuid};
// use catsquad_db::{Db, id_to_string};
use catsquad_log::prelude::*;
// use chrono::DateTime;
use sqlx::{
    Execute,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::env;
use tokio_util::task::TaskTracker;
// use surrealdb::types::ToSql;
use tokio::{io::AsyncWriteExt, time::Instant};

struct BenchArgs {
    pub n: usize,
}

#[tokio::main]
async fn main() {
    init_log();

    let args: Vec<String> = env::args().collect();
    let sample_count = args
        .get(1)
        .and_then(|v| usize::from_str_radix(v, 10).ok())
        .unwrap_or(100);

    let bench_args = BenchArgs { n: sample_count };
    bench_user_add_seq(&bench_args).await;
    bench_user_add_par(&bench_args).await;
    bench_invite_add(&bench_args).await;

    // println!("WRFEFOIENFONE)");
    // trace!("WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW");

    // db.dkk

    // debug!("query result: {result:#?}");

    // let result: String = sqlx::query_as("CREATE DATABASE catsquad2")
    //     .fetch_one(&pool)
    //     .await
    //     .unwrap();

    // let row: (i64,) = sqlx::query_as("SELECT $1")
    //     .bind(150_i64)
    //     .fetch_one(&pool)
    //     .await
    //     .unwrap();

    // assert_eq!(row.0, 150);

    // info!("sample count set: {sample_count}");
    // bench_user_add(sample_count).await;
    // bench_cat_db(sample_count).await;
}

async fn bench_invite_add(args: &BenchArgs) {
    let db = Db::test_db(0, "bench_invite_add").await;

    let n = args.n;

    let emails = (0..n).into_iter().map(|i| format!("prime{i}@heyadora.com"));

    let time = Instant::now();

    for email in emails {
        db.invite_add(0, email, 10).await.unwrap();
    }

    let elapsed = time.elapsed() / n as u32;

    info!("invite_add {elapsed:?}");
}

async fn bench_user_add_par(args: &BenchArgs) {
    let n = args.n;
    let db = Db::test_db(0, "bench_user_add_seq").await;
    let data = bench_user_add_prepare(&db, args).await;
    let itr = data.into_iter();
    let tracker = TaskTracker::new();

    let time = Instant::now();

    for (invite, username) in itr {
        let db = db.clone();
        tracker.spawn(async move {
            db.user_add(0, username, "hey", invite, 10, 10)
                .await
                .unwrap();
        });
    }

    tracker.close();
    tracker.wait().await;

    let elapsed = time.elapsed() / n as u32;

    info!("user_add {elapsed:?}");
}

async fn bench_user_add_seq(args: &BenchArgs) {
    let n = args.n;
    let db = Db::test_db(0, "bench_user_add_seq").await;
    let data = bench_user_add_prepare(&db, args).await;
    let itr = data.into_iter();

    let time = Instant::now();

    for (invite, username) in itr {
        db.user_add(0, username, "hey", invite, 10, 10)
            .await
            .unwrap();
    }

    let elapsed = time.elapsed() / n as u32;

    info!("user_add {elapsed:?}");
}

async fn bench_user_add_prepare(db: &Db, args: &BenchArgs) -> Vec<(Uuid, String)> {
    let n = args.n;
    let mut invites = Vec::with_capacity(n);
    let mut usernames = Vec::with_capacity(n);
    for i in 0..n {
        let email = format!("prime{i}@heyadora.com");
        let username = format!("prime{i}");
        let invite = db.invite_add(2, email, 3).await.unwrap().token;
        invites.push(invite);
        usernames.push(username);
    }

    invites.into_iter().zip(usernames).collect()
}

// async fn bench_cat_db(n: usize) {
//     let mut db = tokio::fs::File::create("/tmp/catsquad-dev/cat.db")
//         .await
//         .unwrap();

//     let time = Instant::now();

//     for _ in 0..n {
//         db.write(b"wtf\n").await.unwrap();
//     }

//     let elapsed = time.elapsed() / n as u32;

//     info!("cat_db {elapsed:?}");
// }
