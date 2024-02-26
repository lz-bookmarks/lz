# The `lz` tagged bookmark manager

When done, this repo will hopefully house a bookmark manager in the style of pinboard.in and del.icio.us. I hope it will be at least as compelling to use, and ideally let you self-host your bookmarks, for your own knowledge-management purposes.

## How this is meant to be used

* This is a project you self-host, in a setup where something else does authentication and authorization.
* Users _exist_ (and are hopefully handled safely), but access control does not.
* The backend uses a sqlite database, which should be speedy and reliable, while being easy to maintain.

## The name

"lz" is short for "Lesezeichen", the german word for "bookmark".

# Grand Designs

I would like this to provide a bit more value than traditional tagged bookmark managers. Here are some ideas:

* Linkding's `notes` field is a great idea. We have that.
* We want to be able to have "related URLs" for bookmarks (maybe even "related bookmarks"?). This might include relevant discussion elsewhere, reddit/HN link posts to the bookmark's URL, web.archive.org wayback history for the URL, and more.
