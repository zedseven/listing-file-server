# listing-file-server
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://choosealicense.com/licenses/mit/)

A feature-equivalent copy of [`rocket::fs::FileServer`](https://api.rocket.rs/v0.5-rc/rocket/fs/struct.FileServer.html)
that provides directory listings when a directory is requested.

Made to be used with [Rocket](https://rocket.rs/) as a replacement for it's
`FileServer`.

Be careful using this in a production environment, as it may expose files
and file structure that you may normally keep hidden. There's a reason it's
commonly advised to disable this feature on most web servers.

This struct uses the same options as its core counterpart, however the
[`rocket::fs::Options::Index`](https://api.rocket.rs/v0.5-rc/rocket/fs/struct.Options.html#associatedconstant.Index)
option has additional considerations.
If enabled, the index file will be served first if available, and directory
listing will only occur if there is no index file.
