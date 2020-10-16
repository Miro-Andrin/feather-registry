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
use rand::Rng;

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

pub async fn download(pool: PgPool, crate_name: String, crate_version: String) -> Result<impl Reply> {
    let dl: String = sqlx::query!(
        "SELECT download FROM crate_version WHERE crate = $1::VARCHAR AND \"version\" = $2::VARCHAR::SEMVER",
        crate_name,
        crate_version,
    )
    .fetch_one(&pool)
    .await
    .unwrap().download;

    Ok(redirect(
        dl.parse::<Uri>()
            .map_err(|_| reject::custom(Error::Internal))?,
    ))
}

#[derive(Debug, Serialize, Deserialize)]
struct Dependency {
    /// Name of the dependency.
    /// If the dependency is renamed from the original package name,
    /// this is the original name. The new package name is stored in
    /// the `explicit_name_in_toml` field.
    name: String,
    /// The semver requirement for this dependency.
    version_req: String,
    /// Array of features (as strings) enabled for this dependency.
    features: Vec<String>,
    /// Boolean of whether or not this is an optional dependency.
    optional: bool,
    /// Boolean of whether or not default features are enabled.
    default_feautres: bool,
    /// The target platform for the dependency.
    /// null if not a target dependency.
    /// Otherwise, a string such as "cfg(windows)".
    target: Option<String>,
    /// The dependency kind.
    /// "dev", "build", or "normal".
    kind: String,
    /// The URL of the index of the registry where this dependency is
    /// from as a string. If not specified or null, it is assumed the
    /// dependency is in the current registry.
    registry: Option<String>,
    /// The URL of the index of the registry where this dependency is
    /// from as a string. If not specified or null, it is assumed the
    /// dependency is in the current registry.
    explicit_name_in_toml: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CrateMeta {
    /// The name of the package.
    name: String,
    /// The version of the package being published.
    vers: String,
    /// Array of direct dependencies of the package.
    deps: Vec<Dependency>,
    /// Set of features defined for the package.
    /// Each feature maps to an array of features or dependencies it enables.
    /// Cargo does not impose limitations on feature names, but crates.io
    /// requires alphanumeric ASCII, `_` or `-` characters.
    features: BTreeMap<String, Vec<String>>,
    /// List of strings of the authors.
    /// May be empty. feather.io requires at least one entry.
    authors: Vec<String>,
    /// Description field from the manifest.
    /// feather.io requires at least some content.
    description: Option<String>,
    /// String of the URL to the website for this package's documentation
    documentation: Option<String>,
    /// String of the URL to the website for this package's home page.
    homepage: Option<String>,
    /// String of the content of the README file.
    readme: Option<String>,
    /// String of a relative path to a README file in the crate.
    readme_file: Option<String>,
    /// Array of strings of keywords for the package.
    keywords: Vec<String>,
    /// Array of strings of categories for the package.
    categories: Vec<String>,
    /// String of the license for the package.
    /// feather.io requires either `license` or `license_file` to be set.
    license: Option<String>,
    /// String of a relative path to a license file in the crate.
    license_file: Option<String>,
    /// String of the URL to the website for the source repository of this package.
    repository: Option<String>,
    /// Optional object of "status" badges. Each value is an object of
    /// arbitrary string to string mappings.
    /// crates.io has special interpretation of the format of the badges.
    badges: BTreeMap<String, BTreeMap<String, String>>,
    /// The `links` string value from the package's manifest, or null if not
    links: Option<String>,
}

impl CrateMeta {
    pub fn get_path(&self) -> path::PathBuf {
        let mut path = path::PathBuf::with_capacity(self.name.len() + 6);
        match self.name.len() {
            0 => panic!(),
            1 => path.push("1"),
            2 => path.push("2"),
            3 => path.push("3"),
            _ => {
                path.push(&self.name[..2]);
                path.push(&self.name[2..4]);
            }
        };
        path
    }
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

    // Path name of git file.
    let path = crate_meta.get_path();
    fs::create_dir_all(&path)
        .await
        .map_err(|_| reject::custom(Error::Internal))?;


    let mut path = path::PathBuf::new();
    path.push("crates");
    path.push(base64::encode(rand::thread_rng().gen::<[u8; 32]>()));
    path.set_extension("crate");

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
