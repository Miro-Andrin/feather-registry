CREATE TABLE "crate"(
    "name" CITEXT NOT NULL,
    "categories" CITEXT[] NOT NULL,
    "keywords" CITEXT[] NOT NULL,
    PRIMARY KEY ("name"),
    CONSTRAINT "valid_name" CHECK ( "name" ~= '[A-Za-z_-][A-Za-z0-9_-]*' )
);

CREATE INDEX "crate_categories_idx" on "crate" USING GIN ("categories");
CREATE INDEX "crate_keywords_idx" on "crate" USING GIN ("keywords");

CREATE TABLE "crate_version"(
    "crate" CITEXT NOT NULL,
    "download" VARCHAR NOT NULL,
    "version" SEMVER NOT NULL,
    PRIMARY KEY ("crate", "version"),
    FOREIGN KEY ("crate") REFERENCES "crate" ("name")
);
