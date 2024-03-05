import { createUseQuery } from "./api";

export function LoadBookmarks() {
  const {
    data: bookmarks,
    error,
    isLoading,
  } = createUseQuery.useFetch("get", "/bookmarks");

  if (isLoading) {
    return <h1>Loading...</h1>;
  }
  if (!bookmarks || error) return <div>An error occurred. Retrying...</div>;

  return (
    <>
      <h1>LZ - bookmarks</h1>
      <ul>
        {bookmarks.map(({ bookmark, tags }) => (
          <li key={bookmark.id}>
            <a href={bookmark.url}>{bookmark.title}</a>
            <div>{bookmark.description}</div>
            <ul className="tags">
              {tags.map((tag) => (
                <li key={tag.name}>{tag.name}</li>
              ))}
            </ul>
            <div>{bookmark.notes}</div>
          </li>
        ))}
      </ul>
    </>
  );
}
