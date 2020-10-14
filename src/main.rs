use dotenv::dotenv;
use http::{uri::Uri, StatusCode};
use std::convert::Infallible;
use tokio::{
    fs, io,
    io::AsyncReadExt,
    stream::{Stream, StreamExt},
    sync::mpsc,
};

use warp::{redirect, reject, Buf, Filter, Reply};

use sqlx::postgres::PgPool;
use std::env;

mod database_structs;
use database_structs::CrateMeta;

#[derive(Debug)]
pub enum Error {
    Internal,
}

impl reject::Reject for Error {}
pub type Result<Ok> = std::result::Result<Ok, reject::Rejection>;

fn routes(pool: PgPool) -> impl Filter<Extract = impl warp::Reply + 'static> + Clone {
    let database = warp::any().map(move || pool.clone());

    let dl = warp::get()
        .and(database.clone())
        .and(warp::path::param())
        .and(warp::path::param())
        .and_then(download);

    let publish = warp::put()
        .and(warp::path("new"))
        .and(database.clone())
        .and(warp::body::stream())
        .and_then(publish);

    let routes = warp::any()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("crates"))
        .and(publish.or(dl))
        .recover(handle_rejection);

    return routes;
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = PgPool::builder()
        .max_size(5)
        .build(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (worker, tx) = Worker::new(pool.clone());
    let routes = routes(pool.clone());

    tokio::spawn(worker.run());

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

struct Worker {
    pool: PgPool,
    rx: mpsc::Receiver<()>,
}

impl Worker {
    fn new(pool: PgPool) -> (Self, mpsc::Sender<()>) {
        let (tx, rx) = mpsc::channel(1);
        (Self { pool, rx }, tx)
    }
    async fn run(mut self) {
        loop {
            let _ = self.rx.recv().await;
            // do the git thing.
        }
    }
}

pub async fn download(pool: PgPool, crate_name: String, version: String) -> Result<impl Reply> {
    let dl: String = sqlx::query!(
        "SELECT download FROM crate_version WHERE crate = $1::VARCHAR AND \"version\" = $2::VARCHAR::SEMVER",
        crate_name,
        version,
    )
    .fetch_one(&pool)
    .await
    .unwrap().download;
    Ok(redirect(
        dl.parse::<Uri>()
            .map_err(|_| reject::custom(Error::Internal))?,
    ))
}

pub async fn publish(
    pool: PgPool,
    stream: impl Stream<Item = std::result::Result<impl Buf, warp::Error>>,
) -> Result<impl Reply> {
    tokio::pin!(stream);
    let stream =
        stream.map(|buffer| buffer.map_err(|err| io::Error::new(io::ErrorKind::Other, err)));
    let mut reader = io::stream_reader(stream);

    let json_length: usize = reader
        .read_u32_le()
        .await
        .map_err(|_| reject::custom(Error::Internal))? as usize;
    let mut json_data = Vec::with_capacity(json_length);
    reader
        .read_exact(&mut json_data[..json_length])
        .await
        .map_err(|_| reject::custom(Error::Internal))?;

    let crate_length = reader
        .read_u32_le()
        .await
        .map_err(|_| reject::custom(Error::Internal))? as u64;
    let mut crate_reader = reader.take(crate_length);

    let crate_meta: CrateMeta =
        serde_json::from_slice(&json_data).map_err(|_| reject::custom(Error::Internal))?;

    sqlx::query!(
        "INSERT INTO crate ( name ) VALUES ( $1::VARCHAR )",
        crate_meta.name
    )
    .execute(&pool)
    .await
    .map_err(|_| reject::custom(Error::Internal))?;

    let path = crate_meta.get_path();
    fs::create_dir_all(&path)
        .await
        .map_err(|_| reject::custom(Error::Internal))?;

    let mut file = fs::File::create(&path)
        .await
        .map_err(|_| reject::custom(Error::Internal))?;
    io::copy(&mut crate_reader, &mut file)
        .await
        .map_err(|_| reject::custom(Error::Internal))?;

    Ok(warp::reply::json(&"ok"))
}

pub fn handle_error(result: Result<impl Reply>) -> impl Reply {
    result.unwrap()
}

async fn handle_rejection(
    _err: reject::Rejection,
) -> ::std::result::Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("error", StatusCode::OK))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish() {
        // let routes = routes();
    }
}
