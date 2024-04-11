//
// From https://leanrada.com/htmz/extensions/:
function htmz(frame) {
  // ---------------------------------8<-----------------------------------
  // No history
  // ----------------------------------------------------------------------
  // This extension clears the iframe's history from the global history
  // by removing the iframe from the DOM (but immediately adding it back
  // for subsequent requests).
  if (frame.contentWindow.location.href === "about:blank") return;
  // --------------------------------->8-----------------------------------
  setTimeout(() => {
    // push the new URL to browser history
    const loc = frame.contentWindow.location;
    window.history.pushState({}, "", loc.pathname + loc.search);

    // Assign elements and replace iframe to avoid history modification:
    document
      .querySelector(frame.contentWindow.location.hash || null)
      ?.replaceWith(...frame.contentDocument.body.childNodes);
    frame.remove();
    document.body.appendChild(frame);
  });
}
