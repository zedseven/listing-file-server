// Uses
use std::{
	cmp::Reverse,
	fs::read_dir,
	path::{Path, PathBuf},
};

use rocket::{
	async_trait,
	error,
	figment,
	fs::{NamedFile, Options},
	http::{ext::IntoOwned, uri::Segments, Method},
	response::Redirect,
	route::{Handler, Outcome, Route},
	warn_,
	Data,
	Request,
};
use rocket_dyn_templates::Template;

/// A feature-equivalent copy of [`rocket::fs::FileServer`] that provides
/// directory listings when a directory is requested.
///
/// Be careful using this in a production environment, as it may expose files
/// and file structure that you may normally keep hidden. There's a reason it's
/// commonly advised to disable this feature on most web servers.
///
/// This struct uses the same options as it's core counterpart, however the
/// [`rocket::fs::Options::Index`] option has additional considerations.
/// If enabled, the index file will be served first if available, and directory
/// listing will only occur if there is no index file.
#[derive(Debug, Clone)]
pub struct ListingFileServer<R: 'static + Fn(String, Vec<String>) -> Template + Send + Sync + Clone>
{
	root: PathBuf,
	options: Options,
	rank: isize,
	template_renderer: R,
}

impl<R: 'static + Fn(String, Vec<String>) -> Template + Send + Sync + Clone> ListingFileServer<R> {
	/// The default rank use by `FileServer` routes.
	const DEFAULT_RANK: isize = 10;

	/// Creates an instance of [`ListingFileServer`] with a path, no options
	/// enabled (different from [`rocket::fs::FileServer`]), and a
	/// template-rendering function.
	///
	/// The reason the default here doesn't have [`rocket::fs::Options::Index`]
	/// enabled is because it goes against what this struct is made to do. If
	/// using this type over [`rocket::fs::FileServer`], directory listing is
	/// the expected default behaviour.
	///
	/// The template renderer receives a list of filenames found within the
	/// directory, expected to be used in relative links.
	#[track_caller]
	pub fn from<P>(path: P, template_renderer: R) -> Self
	where
		P: AsRef<Path>,
		R: 'static + Fn(String, Vec<String>) -> Template + Send + Sync + Clone,
	{
		ListingFileServer::new(path, Options::None, template_renderer)
	}

	/// Creates an instance of [`ListingFileServer`] with a path, options, and a
	/// template-rendering function.
	///
	/// The template renderer receives a list of filenames found within the
	/// directory, expected to be used in relative links.
	#[track_caller]
	pub fn new<P>(path: P, options: Options, template_renderer: R) -> Self
	where
		P: AsRef<Path>,
		R: 'static + Fn(String, Vec<String>) -> Template + Send + Sync + Clone,
	{
		use rocket::yansi::Paint;

		let path = path.as_ref();
		if !path.is_dir() {
			let path = path.display();
			error!(
				"ListingFileServer path '{}' is not a directory.",
				Paint::white(path)
			);
			warn_!("Aborting early to prevent inevitable handler failure.");
			panic!("bad ListingFileServer path: refusing to continue");
		}

		ListingFileServer {
			root: path.into(),
			options,
			rank: Self::DEFAULT_RANK,
			template_renderer,
		}
	}

	/// Sets the rank for generated routes to `rank`.
	pub fn rank(mut self, rank: isize) -> Self {
		self.rank = rank;
		self
	}
}

impl<R: 'static + Fn(String, Vec<String>) -> Template + Send + Sync + Clone> Into<Vec<Route>>
	for ListingFileServer<R>
{
	fn into(self) -> Vec<Route> {
		let source = figment::Source::File(self.root.clone());
		let mut route = Route::ranked(self.rank, Method::Get, "/<path..>", self);
		route.name = Some(format!("ListingFileServer: {}/", source).into());
		vec![route]
	}
}

#[async_trait]
impl<R: 'static + Fn(String, Vec<String>) -> Template + Send + Sync + Clone> Handler
	for ListingFileServer<R>
{
	async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
		use rocket::http::uri::fmt::Path;

		// Get the segments as a `PathBuf`, allowing dotfiles requested.
		let options = self.options;
		let allow_dotfiles = options.contains(Options::DotFiles);
		let req_path = req
			.segments::<Segments<'_, Path>>(0..)
			.ok()
			.and_then(|segments| segments.to_path_buf(allow_dotfiles).ok());
		let path = req_path.clone().map(|path| self.root.join(path));

		match path {
			Some(p) if p.is_dir() => {
				// Normalize '/a/b/foo' to '/a/b/foo/'.
				if options.contains(Options::NormalizeDirs) && !req.uri().path().ends_with('/') {
					let normal = req
						.uri()
						.map_path(|p| format!("{}/", p))
						.expect("adding a trailing slash to a known good path => valid path")
						.into_owned();

					return Outcome::from_or_forward(req, data, Redirect::permanent(normal));
				}

				if options.contains(Options::Index) {
					let index = NamedFile::open(p.join("index.html")).await.ok();
					if index.is_some() {
						return Outcome::from(req, index);
					}
				}

				match read_dir(&p) {
					// Directory
					Ok(dir_entries) => {
						// Prepare the directory path string
						let mut directory = String::from('/');
						directory.push_str(
							req_path
								.unwrap()
								.into_os_string()
								.into_string()
								.expect("Unable to convert directory path from OS string")
								.replace('\\', "/")
								.as_str(),
						);
						if !directory.ends_with('/') {
							directory.push('/');
						}
						// Prepare the directory entries list
						let mut entry_list = dir_entries
							.filter(|res| res.is_ok())
							.map(|res| {
								let mut entry = res
									.unwrap()
									.file_name()
									.into_string()
									.expect("Unable to convert directory entry from OS string");
								let is_dir = p.join(&entry).is_dir();
								if is_dir {
									entry.push('/');
								}
								(Reverse(is_dir), entry)
							})
							.collect::<Vec<_>>();
						entry_list.sort_unstable();
						// Render the template
						Outcome::from(
							req,
							(self.template_renderer)(
								directory,
								entry_list.drain(..).map(|e| e.1).collect::<Vec<_>>(),
							),
						)
					}
					// File
					_ => Outcome::forward(data),
				}
			}
			Some(p) => Outcome::from_or_forward(req, data, NamedFile::open(p).await.ok()),
			None => Outcome::forward(data),
		}
	}
}
