-- Add migration script here

CREATE TABLE "bookmarks" (
  "bookmark_id" INTEGER NOT NULL PRIMARY KEY,
  "url" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "description" TEXT,
  "website_title" TEXT,
  "website_description" TEXT,
  "notes" TEXT
);

CREATE TABLE "tags" (
  "tag_id" INTEGER NOT NULL PRIMARY KEY,
  "name" TEXT NOT NULL UNIQUE
);

CREATE UNIQUE INDEX "tags_by_name" ON "tags" ("name");

CREATE TABLE "bookmark_tags" (
  "bookmark_id" INTEGER NOT NULL,
  "tag_id" INTEGER NOT NULL,

  PRIMARY KEY ("bookmark_id","tag_id"),
  FOREIGN KEY("bookmark_id") REFERENCES "bookmarks"("bookmark_id"),
  FOREIGN KEY("tag_id") REFERENCES "tags"("tag_id")
);
