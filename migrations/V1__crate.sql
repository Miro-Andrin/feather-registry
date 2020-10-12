CREATE TABLE "crate"(
    "name" CITEXT NOT NULL,
    "owner" INT NOT NULL,
    PRIMARY KEY ("name"),
    FOREIGN KEY ("owner") REFERENCES "user" ("id"),
    CONSTRAINT "valid_name" CHECK ( "name" ~= '[A-Za-z_-][A-Za-z0-9_-]*' )
);

CREATE TABLE "crate_version"(
    "crate" CITEXT NOT NULL,
    "download" VARCHAR NOT NULL,
    "version" SEMVER NOT NULL,
    "authors" VARCHAR[] NOT NULL,
    "description" VARCHAR NOT NULL,
    "documentation" VARCHAR NULL,
    "homepage" VARCHAR NULL,
    "readme" VARCHAR NULL,
    "readme_file" VARCHAR NULL,
    "categories" CITEXT[] NOT NULL,
    "keywords" CITEXT[] NOT NULL,
    "license" VARCHAR NULL,
    "license_file" VARCHAR NULL,
    "repository" VARCHAR NULL,
    "links" VARCHAR NULL,
    PRIMARY KEY ("crate", "version"),
    FOREIGN KEY ("crate") REFERENCES "crate" ("name"),
    CONSTRAINT "license_present" CHECK ( "license" IS NOT NULL OR "license_file" IS NOT NULL )
);

CREATE INDEX "crate_version_categories_idx" ON "crate_version" USING GIN ("categories");
CREATE INDEX "crate_version_keywords_idx" ON "crate_version" USING GIN ("keywords");
