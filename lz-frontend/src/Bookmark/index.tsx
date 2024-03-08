import type { components } from "../api/v1.d.ts";
import "./index.css";
import { Link, useLocation } from "react-router-dom";

interface TagSet {
  [key: string]: boolean | undefined;
}

export function Bookmark({
  bookmark,
  tags,
}: components["schemas"]["AnnotatedBookmark"]) {
  const location = useLocation();
  const onTagPage = location.pathname.match(/^\/tag\//);
  const existingTags: TagSet = {};
  if (onTagPage) {
    const tagQuery = location.pathname.split("/", 3).slice(-1)[0] || "";
    for (const tag of tagQuery.split(" ")) {
      existingTags[tag] = true;
    }
  }
  return (
    <article key={bookmark.id}>
      <a href={bookmark.url}>{bookmark.title}</a>
      {bookmark.description && bookmark.description !== "" && (
        <div>{bookmark.description}</div>
      )}
      <ul className="tags">
        {tags.map(({ name }) => (
          <li key={name}>
            <Link
              to={{
                pathname: `/tag/${Object.keys({
                  ...existingTags,
                  [name]: true,
                }).join(" ")}`,
              }}
            >
              {name}
            </Link>
          </li>
        ))}
      </ul>
      {bookmark.notes && <blockquote>{bookmark.notes}</blockquote>}
    </article>
  );
}
