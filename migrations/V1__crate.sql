CREATE TABLE crate(
    name CITEXT NOT NULL,
    PRIMARY KEY (name),
    CONSTRAINT valid_name CHECK ( name ~= '[A-Za-z_-][A-Za-z0-9_-]*' )
);

CREATE TABLE crate_version(
    crate CITEXT NOT NULL,
    download VARCHAR NOT NULL,
    "version" SEMVER NOT NULL,
    PRIMARY KEY (crate, "version"),
    FOREIGN KEY (crate) REFERENCES crate (name)
);