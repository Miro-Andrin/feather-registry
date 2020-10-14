
/* IF NOT EXISTS requires postgres 9.1 or greater */
CREATE TABLE IF NOT EXISTS "user"(
    "id" SERIAL NOT NULL,
     PRIMARY KEY("id")
);