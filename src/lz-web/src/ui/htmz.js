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
    document
      .querySelector(frame.contentWindow.location.hash || null)
      ?.replaceWith(...frame.contentDocument.body.childNodes);
    // ---------------------------------8<-----------------------------------
    frame.remove();
    document.body.appendChild(frame);
    // --------------------------------->8-----------------------------------
  });
}
