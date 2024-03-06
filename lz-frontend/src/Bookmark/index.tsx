import type { components } from "../api/v1.d.ts";
import "./index.css";
import { Link } from "react-router-dom";

export function Bookmark({
  bookmark,
  tags,
}: components["schemas"]["AnnotatedBookmark"]) {
  return (
    <article key={bookmark.id}>
      <a href={bookmark.url}>{bookmark.title}</a>
      {bookmark.description && bookmark.description !== "" && (
        <div>{bookmark.description}</div>
      )}
      <ul className="tags">
        {tags.map((tag) => (
          <li key={tag.name}>
            <Link to={`/tag/${tag.name}`}>{tag.name}</Link>
          </li>
        ))}
      </ul>
      {bookmark.notes && <blockquote>{bookmark.notes}</blockquote>}
    </article>
  );
}