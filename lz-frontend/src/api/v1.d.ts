/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */


export interface paths {
  "/bookmarks": {
    /**
     * List the user's bookmarks, newest to oldest.
     * @description List the user's bookmarks, newest to oldest.
     */
    get: operations["list_bookmarks"];
  };
  "/bookmarks/tagged/{query}": {
    /**
     * List bookmarks matching a tag, newest to oldest.
     * @description List bookmarks matching a tag, newest to oldest.
     */
    get: operations["list_bookmarks_with_tag"];
  };
}

export type webhooks = Record<string, never>;

export interface components {
  schemas: {
    /** @description A bookmark, including tags set on it. */
    AnnotatedBookmark: {
      bookmark: components["schemas"]["ExistingBookmark"];
      tags: components["schemas"]["ExistingTag"][];
    };
    /**
     * Format: int64
     * @description The database ID of a bookmark.
     */
    BookmarkId: number;
    /**
     * @description A bookmark saved by a user.
     *
     * See the section in [Transaction][Transaction#working-with-bookmarks]
     */
    ExistingBookmark: {
      /**
       * Format: date-time
       * @description Last time the bookmark was accessed via the web
       */
      accessed_at?: string | null;
      /**
       * Format: date-time
       * @description Time at which the bookmark was created.
       *
       * This time is assigned in code here, not in the database.
       */
      created_at: string;
      /** @description Description of the bookmark, possibly extracted from the website. */
      description?: string | null;
      id: components["schemas"]["BookmarkId"];
      /**
       * Format: date-time
       * @description Last time the bookmark was modified.
       *
       * This field indicates modifications to the bookmark data itself
       * only, not changes to tags or related models.
       */
      modified_at?: string | null;
      /** @description Private notes that the user attached to the bookmark. */
      notes?: string | null;
      /** @description Whether other users can see the bookmark. */
      shared: boolean;
      /** @description Title that the user gave the bookmark. */
      title: string;
      /** @description Whether the bookmark is "to read" */
      unread: boolean;
      /**
       * Format: uri
       * @description URL that the bookmark points to.
       */
      url: string;
      user_id: components["schemas"]["UserId"];
      /** @description Original description extracted from the website. */
      website_description?: string | null;
      /** @description Original title extracted from the website. */
      website_title?: string | null;
    };
    /**
     * @description A named tag, possibly assigned to multiple bookmarks.
     *
     * See the section in [Transaction][Transaction#working-with-tags]
     */
    ExistingTag: {
      /**
       * Format: date-time
       * @description When the tag was first created.
       */
      created_at: string;
      /** @description Name of the tag. */
      name: string;
    };
    /**
     * @description The response returned by the `list_bookmarks` API endpoint.
     *
     * This response contains pagination information; if `next_cursor` is
     * set, passing that value to the `cursor` pagination parameter will
     * fetch the next page.
     */
    ListBookmarkResult: {
      bookmarks: components["schemas"]["AnnotatedBookmark"][];
      nextCursor?: components["schemas"]["BookmarkId"] | null;
    };
    /**
     * @description Parameters that govern non-offset based pagination.
     *
     * Pagination in `lz` works by getting the next page based on what
     * the previous page's last element was, aka "cursor-based
     * pagination". To that end, use the previous call's `nextCursor`
     * parameter into this call's `cursor` parameter.
     */
    Pagination: {
      /** @default null */
      cursor?: components["schemas"]["BookmarkId"] | null;
      /**
       * Format: int32
       * @description How many items to return
       * @default null
       * @example 50
       */
      perPage?: number | null;
    };
    TagName: string;
    /**
     * @description A search query for retrieving bookmarks via the tags assigned to them.
     *
     * These tag queries are made in a URL path, separated by space
     * (`%20`) characters.
     */
    TagQuery: {
      /** @description Tags that all returned items should have. */
      tags: components["schemas"]["TagName"][];
    };
    /**
     * Format: int64
     * @description The database ID of a user.
     */
    UserId: number;
  };
  responses: {
    /** @description A bookmark, including tags set on it. */
    AnnotatedBookmark: {
      content: {
        "application/json": {
          bookmark: components["schemas"]["ExistingBookmark"];
          tags: components["schemas"]["ExistingTag"][];
        };
      };
    };
    /**
     * @description A bookmark saved by a user.
     *
     * See the section in [Transaction][Transaction#working-with-bookmarks]
     */
    Bookmark: {
      content: {
        "application/json": {
          /**
           * Format: date-time
           * @description Last time the bookmark was accessed via the web
           */
          accessed_at?: string | null;
          /**
           * Format: date-time
           * @description Time at which the bookmark was created.
           *
           * This time is assigned in code here, not in the database.
           */
          created_at: string;
          /** @description Description of the bookmark, possibly extracted from the website. */
          description?: string | null;
          id: components["schemas"]["ID"];
          /**
           * Format: date-time
           * @description Last time the bookmark was modified.
           *
           * This field indicates modifications to the bookmark data itself
           * only, not changes to tags or related models.
           */
          modified_at?: string | null;
          /** @description Private notes that the user attached to the bookmark. */
          notes?: string | null;
          /** @description Whether other users can see the bookmark. */
          shared: boolean;
          /** @description Title that the user gave the bookmark. */
          title: string;
          /** @description Whether the bookmark is "to read" */
          unread: boolean;
          /**
           * Format: uri
           * @description URL that the bookmark points to.
           */
          url: string;
          user_id: components["schemas"]["UID"];
          /** @description Original description extracted from the website. */
          website_description?: string | null;
          /** @description Original title extracted from the website. */
          website_title?: string | null;
        };
      };
    };
    /**
     * @description The response returned by the `list_bookmarks` API endpoint.
     *
     * This response contains pagination information; if `next_cursor` is
     * set, passing that value to the `cursor` pagination parameter will
     * fetch the next page.
     */
    ListBookmarkResult: {
      content: {
        "application/json": {
          bookmarks: components["schemas"]["AnnotatedBookmark"][];
          nextCursor?: components["schemas"]["BookmarkId"] | null;
        };
      };
    };
    /**
     * @description A named tag, possibly assigned to multiple bookmarks.
     *
     * See the section in [Transaction][Transaction#working-with-tags]
     */
    Tag: {
      content: {
        "application/json": {
          /**
           * Format: date-time
           * @description When the tag was first created.
           */
          created_at: string;
          /** @description Name of the tag. */
          name: string;
        };
      };
    };
    /** @description The database ID of a user. */
    UserId: {
      content: {
        "text/plain": number;
      };
    };
  };
  parameters: never;
  requestBodies: never;
  headers: never;
  pathItems: never;
}

export type $defs = Record<string, never>;

export type external = Record<string, never>;

export interface operations {

  /**
   * List the user's bookmarks, newest to oldest.
   * @description List the user's bookmarks, newest to oldest.
   */
  list_bookmarks: {
    parameters: {
      query?: {
        pagination?: ({
          /** @default null */
          cursor?: components["schemas"]["BookmarkId"] | null;
          /**
           * Format: int32
           * @description How many items to return
           * @default null
           * @example 50
           */
          perPage?: number | null;
        }) | null;
      };
    };
    responses: {
      /** @description Lists all bookmarks */
      200: {
        content: {
          "application/json": {
            bookmarks: components["schemas"]["AnnotatedBookmark"][];
            nextCursor?: components["schemas"]["BookmarkId"] | null;
          };
        };
      };
    };
  };
  /**
   * List bookmarks matching a tag, newest to oldest.
   * @description List bookmarks matching a tag, newest to oldest.
   */
  list_bookmarks_with_tag: {
    parameters: {
      query?: {
        pagination?: ({
          /** @default null */
          cursor?: components["schemas"]["BookmarkId"] | null;
          /**
           * Format: int32
           * @description How many items to return
           * @default null
           * @example 50
           */
          perPage?: number | null;
        }) | null;
      };
      path: {
        query: string;
      };
    };
    responses: {
      /** @description Lists bookmarks matching the tag */
      200: {
        content: {
          "application/json": {
            bookmarks: components["schemas"]["AnnotatedBookmark"][];
            nextCursor?: components["schemas"]["BookmarkId"] | null;
          };
        };
      };
    };
  };
}
