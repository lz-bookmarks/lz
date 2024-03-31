-- Add migration script here

CREATE TABLE "users" (
  "user_id" INTEGER NOT NULL PRIMARY KEY,
  "name" TEXT NOT NULL UNIQUE,
  "created_at" TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX "users_by_name" ON "users" ("name");

CREATE TABLE "bookmarks" (
  "bookmark_id" INTEGER NOT NULL PRIMARY KEY,
  "user_id" INTEGER NOT NULL,
  "created_at" TEXT NOT NULL,
  "modified_at" TEXT,
  "accessed_at" TEXT,
  "url" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "description" TEXT,
  "website_title" TEXT,
  "website_description" TEXT,
  "notes" TEXT,
  "unread" BOOLEAN,
  "shared" BOOLEAN,
  "primary_link" INTEGER,
  "import_properties" TEXT,

  FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
) STRICT;

CREATE UNIQUE INDEX "bookmarks_by_user_and_url" ON "bookmarks" ("user_id", "url");

CREATE TABLE "tags" (
  "tag_id" INTEGER NOT NULL PRIMARY KEY,
  "created_at" TEXT NOT NULL,
  "name" TEXT NOT NULL UNIQUE,

  CHECK("name" NOT LIKE '% %' AND length("name") >= 1)
) STRICT;

CREATE UNIQUE INDEX "tags_by_name" ON "tags" ("name");

CREATE TABLE "bookmark_tags" (
  "bookmark_id" INTEGER NOT NULL,
  "tag_id" INTEGER NOT NULL,

  PRIMARY KEY ("bookmark_id","tag_id"),
  FOREIGN KEY("bookmark_id") REFERENCES "bookmarks"("bookmark_id"),
  FOREIGN KEY("tag_id") REFERENCES "tags"("tag_id")
) STRICT;

CREATE TABLE "bookmark_associations" (
  "primary_bookmark_id" INTEGER NOT NULL,
  "secondary_bookmark_id" INTEGER NOT NULL,
  "context" TEXT,

  PRIMARY KEY ("primary_bookmark_id","secondary_bookmark_id"),
  FOREIGN KEY ("primary_bookmark_id") REFERENCES "bookmarks"("bookmark_id"),
  FOREIGN KEY ("secondary_bookmark_id") REFERENCES "bookmarks"("bookmark_id")
) STRICT;
