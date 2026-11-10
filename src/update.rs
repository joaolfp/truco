//! Checks GitHub for a newer release and installs it over the current one,
//! so players never have to remove the app and download it again by hand.
//!
//! Each platform updates in place with the installer it was shipped with
//! (see .github/workflows/build-installers.yml for the asset names):
//! - macOS: the .app bundle is replaced by the one inside the new .dmg.
//! - Linux AppImage: the AppImage file is replaced by the new one.
//! - Windows: the new NSIS installer runs over the existing install.
//! - Linux .deb: the new package opens in the system's package installer.
//!
//! Anything else (e.g. `cargo run`) just opens the release page.

use std::fs;
use std::future::Future;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use iced::futures::channel::oneshot;
use serde::Deserialize;

/// Version of the running app, e.g. "0.2.0".
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPOSITORY: &str = "joaolfp/truco";

/// A release newer than the running app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
	/// Git tag, e.g. "v0.3.0".
	pub tag: String,
}

impl Release {
	/// Version without the leading "v", e.g. "0.3.0".
	pub fn version(&self) -> &str {
		self.tag.trim_start_matches('v')
	}

	fn download_url(&self, asset: &str) -> String {
		format!(
			"https://github.com/{REPOSITORY}/releases/download/{}/{asset}",
			self.tag
		)
	}

	fn page_url(&self) -> String {
		format!("https://github.com/{REPOSITORY}/releases/tag/{}", self.tag)
	}
}

/// What happened after an update was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Installed {
	/// The new version is starting (or its installer is running):
	/// this process should exit.
	Restart,
	/// The download page or the system package installer was opened;
	/// the player finishes the update there.
	Handed,
}

/// Asks GitHub for the latest release. `Ok(None)` means up to date.
pub fn check() -> impl Future<Output = Result<Option<Release>, String>> {
	in_background(|| {
		#[derive(Deserialize)]
		struct Latest {
			tag_name: String,
		}

		let latest: Latest = agent()
			.get(format!(
				"https://api.github.com/repos/{REPOSITORY}/releases/latest"
			))
			.header("Accept", "application/vnd.github+json")
			.call()
			.and_then(|mut response| response.body_mut().read_json())
			.map_err(|error| error.to_string())?;

		let release = Release {
			tag: latest.tag_name,
		};
		Ok(is_newer(release.version(), CURRENT_VERSION).then_some(release))
	})
}

/// Downloads `release` and installs it over the running app.
pub fn install(release: Release) -> impl Future<Output = Result<Installed, String>> {
	in_background(move || install_blocking(&release).map_err(|error| error.to_string()))
}

/// `true` when `candidate` (e.g. "0.10.0") is a later version than `current`.
fn is_newer(candidate: &str, current: &str) -> bool {
	fn parse(version: &str) -> Option<[u64; 3]> {
		// Ignore pre-release/build suffixes such as "-beta.1".
		let core = version.split(['-', '+']).next()?;
		let mut parts = core.split('.').map(str::parse::<u64>);
		let parsed = [
			parts.next()?.ok()?,
			parts.next()?.ok()?,
			parts.next()?.ok()?,
		];
		parts.next().is_none().then_some(parsed)
	}
	matches!((parse(candidate), parse(current)), (Some(a), Some(b)) if a > b)
}

fn agent() -> ureq::Agent {
	ureq::Agent::config_builder()
		.user_agent(concat!("truco/", env!("CARGO_PKG_VERSION")))
		.build()
		.into()
}

/// Runs blocking network and disk work off the UI thread.
fn in_background<T: Send + 'static>(
	work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> impl Future<Output = Result<T, String>> {
	let (sender, receiver) = oneshot::channel();
	std::thread::spawn(move || {
		let _ = sender.send(work());
	});
	async move {
		receiver
			.await
			.unwrap_or_else(|_| Err("update worker stopped unexpectedly".into()))
	}
}

/// Downloads `url` into `destination`, replacing any previous file.
fn download(url: &str, destination: &Path) -> io::Result<()> {
	let mut response = agent().get(url).call().map_err(io::Error::other)?;
	let mut file = fs::File::create(destination)?;
	io::copy(&mut response.body_mut().as_reader(), &mut file)?;
	file.sync_all()
}

/// A fresh temporary folder for the downloaded installer.
fn work_dir() -> io::Result<PathBuf> {
	let dir = std::env::temp_dir().join(format!("truco-update-{}", std::process::id()));
	if dir.exists() {
		fs::remove_dir_all(&dir)?;
	}
	fs::create_dir_all(&dir)?;
	Ok(dir)
}

// Unused on Windows and Linux.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn run(command: &mut Command) -> io::Result<()> {
	let output = command.output()?;
	if output.status.success() {
		Ok(())
	} else {
		Err(io::Error::other(format!(
			"{:?} failed: {}",
			command.get_program(),
			String::from_utf8_lossy(&output.stderr).trim()
		)))
	}
}

/// Opens a URL or file with the system's default handler.
#[cfg_attr(target_os = "windows", allow(dead_code))]
fn open(target: &str) -> io::Result<()> {
	let mut command = if cfg!(target_os = "windows") {
		let mut command = Command::new("cmd");
		command.args(["/C", "start", ""]);
		command
	} else if cfg!(target_os = "macos") {
		Command::new("open")
	} else {
		Command::new("xdg-open")
	};
	command.arg(target).spawn().map(drop)
}

#[cfg(target_os = "macos")]
fn install_blocking(release: &Release) -> io::Result<Installed> {
	// .../Truco.app/Contents/MacOS/truco -> .../Truco.app
	let exe = std::env::current_exe()?;
	let Some(bundle) = exe
		.ancestors()
		.nth(3)
		.filter(|path| path.extension().is_some_and(|ext| ext == "app"))
	else {
		open(&release.page_url())?;
		return Ok(Installed::Handed);
	};

	let dir = work_dir()?;
	let dmg = dir.join("Truco-macOS.dmg");
	download(&release.download_url("Truco-macOS.dmg"), &dmg)?;

	let mount = dir.join("mount");
	run(
		Command::new("hdiutil")
			.args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
			.arg(&mount)
			.arg(&dmg),
	)?;
	let copied = replace_bundle(&mount, bundle);
	let _ = run(
		Command::new("hdiutil")
			.arg("detach")
			.arg(&mount)
			.arg("-force"),
	);
	copied?;
	let _ = fs::remove_dir_all(&dir);

	// Start the new version once this one has quit.
	Command::new("open").arg("-n").arg(bundle).spawn()?;
	Ok(Installed::Restart)
}

/// Copies the .app from the mounted image over `bundle`, swapping it in only
/// after the copy succeeded so a failed update leaves the old app intact.
#[cfg(target_os = "macos")]
fn replace_bundle(mount: &Path, bundle: &Path) -> io::Result<()> {
	let new_app = fs::read_dir(mount)?
		.filter_map(Result::ok)
		.map(|entry| entry.path())
		.find(|path| path.extension().is_some_and(|ext| ext == "app"))
		.ok_or_else(|| io::Error::other("no .app found in the downloaded image"))?;

	let staged = bundle.with_extension("app.new");
	let old = bundle.with_extension("app.old");
	for leftover in [&staged, &old] {
		if leftover.exists() {
			fs::remove_dir_all(leftover)?;
		}
	}
	run(Command::new("ditto").arg(&new_app).arg(&staged))?;
	fs::rename(bundle, &old)?;
	if let Err(error) = fs::rename(&staged, bundle) {
		let _ = fs::rename(&old, bundle);
		return Err(error);
	}
	let _ = fs::remove_dir_all(&old);
	Ok(())
}

#[cfg(target_os = "windows")]
fn install_blocking(release: &Release) -> io::Result<Installed> {
	let dir = work_dir()?;
	let setup = dir.join("Truco-Windows-Setup.exe");
	download(&release.download_url("Truco-Windows-Setup.exe"), &setup)?;
	// The installer upgrades the existing install; the app quits right after
	// so its files are free to be replaced.
	Command::new(&setup).spawn()?;
	Ok(Installed::Restart)
}

#[cfg(target_os = "linux")]
fn install_blocking(release: &Release) -> io::Result<Installed> {
	use std::os::unix::fs::PermissionsExt;

	// Set by the AppImage runtime to the path of the .AppImage file.
	if let Some(appimage) = std::env::var_os("APPIMAGE").map(PathBuf::from) {
		let staged = appimage.with_extension("AppImage.new");
		download(
			&release.download_url("Truco-Linux-x86_64.AppImage"),
			&staged,
		)?;
		fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))?;
		// Replacing a running executable is fine on Linux: this process keeps
		// the old inode until it exits.
		fs::rename(&staged, &appimage)?;
		Command::new(&appimage).spawn()?;
		return Ok(Installed::Restart);
	}

	// Installed from the .deb: hand the new package to the system installer,
	// which upgrades it in place.
	if std::env::current_exe()?.starts_with("/usr") {
		let dir = work_dir()?;
		let deb = dir.join("Truco-Linux-amd64.deb");
		download(&release.download_url("Truco-Linux-amd64.deb"), &deb)?;
		open(&deb.to_string_lossy())?;
		return Ok(Installed::Handed);
	}

	open(&release.page_url())?;
	Ok(Installed::Handed)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn install_blocking(release: &Release) -> io::Result<Installed> {
	open(&release.page_url())?;
	Ok(Installed::Handed)
}

#[cfg(test)]
#[path = "../tests/update.rs"]
mod tests;
