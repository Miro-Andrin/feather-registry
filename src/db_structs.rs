use sqlx::{query, query_as};


use sqlx::{PgPool};

struct Crate<'a> {
    name: &'a str,
    owner: i64,
}
// crate_version  db
#[derive(sqlx::FromRow,Debug)]
pub struct CrateVersion {
    pub crate_name: String,
    pub download: String,
    pub version: String, // Should maybe be a semver type
    pub authors: Vec<String>,
    pub description: String,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub links: Option<String>,
    pub uploaded_at: String, //Should maybe be a datetime type?
    // This entry is null if the crate version has never been commited.
    pub git_hash: Option<Vec<u8>>, // BYTEA NULL,
}

impl CrateVersion {

    pub async fn all_not_pushed(pool: &PgPool) -> Result<Vec<CrateVersion>,  Box<dyn std::error::Error>> {
        let rows = sqlx::query_as!(
            CrateVersion,
            "
            SELECT 
                CAST(crate AS VARCHAR) AS crate_name,
                download,
                CAST(version AS VARCHAR),
                authors,
                description,
                documentation, 
                homepage,
                readme,
                readme_file, 
                CAST(categories AS VARCHAR[]),
                CAST(keywords AS VARCHAR[]),
                license,
                license_file, 
                repository,
                links,
                CAST(uploaded_at AS VARCHAR),
                git_hash
            FROM 
                crate_version
            WHERE 
                git_hash IS NULL;"
        ).fetch_all(pool).await?;



        Ok(rows)
    }
}
