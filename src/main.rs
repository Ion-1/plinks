#![feature(explicit_tail_calls)]
#![expect(incomplete_features)]

use crate::app::AppCache;
use crate::profile_loader::{Browsers, Installation};
use clap::Parser;
use log::{debug, error, info, trace, warn};
use std::io::Write;
use std::path::{Path, PathBuf};
use panic::setup_panic;

mod app;
pub mod profile_loader;
pub mod ui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    uri: Option<String>,
}

fn main() {
    setup_panic! {
        name: "Panic Wrapper",
        short_name: "panic",
        version: env!("CARGO_PKG_VERSION"),
        repository: "https://github.com/exact-labs/panic",
        messages: {
            colors: (Color::Red, Color::White, Color::Green),
            head: "Well, this is embarrassing. %(name) v%(version) had a problem and crashed. \nTo help us diagnose the problem you can send us a crash report\n",
            body: "We have generated a report file at \"%(file_path)\". \nSubmit an issue or email with the subject of \"%(name) v%(version) crash report\" and include the report as an attachment at %(repository).\n",
            footer: "We take privacy seriously, and do not perform any automated error collection. \nIn order to improve the software, we rely on people to submit reports. Thank you!"
        }
    };

    let cli = Args::parse();

    if let Some(uri) = cli.uri {
        if let Err(err) = std::fs::File::create(
            std::env::current_exe()
                .unwrap() // TODO Error handling
                .parent()
                .unwrap() // current_exe should never be root, unwrap is fine
                .join("config\\last_url.txt"),
        )
        .and_then(|mut f| f.write(uri.as_bytes()))
        {
            error!("Failed to write last url with error: {}", err);
        }
        let mut cache: AppCache = load_cache();
        #[allow(clippy::zombie_processes)] // Ideally we are detaching the new process
        ui::open_dialog(uri, &mut cache.installations)
            .unwrap() // TODO Error handling
            .spawn()
            .expect("panic message"); // TODO Error handling
        save_cache(&cache);
    } else {
        // let mut cache: AppCache = load_cache();
        let cache = AppCache {
            cfg_version: 0,
            installations: vec![
                Installation::from_installation_path(
                    PathBuf::from(r"C:\Program Files\Mozilla Firefox\").as_ref(),
                )
                .unwrap(),
                Installation::from_installation_path(
                    PathBuf::from(r"C:\Portables\LibreWolfPortable\").as_ref(),
                )
                .unwrap(),
            ],
        };
        save_cache(&cache);
        // ui::open_config(&cache.installations);
    }
}

#[cfg(feature = "portable")]
fn load_cache() -> AppCache {
    confy::load_path(
        std::env::current_exe()
            .unwrap() // TODO Error handling
            .parent()
            .unwrap() // current_exe should never be root, unwrap is fine
            .join("config\\cache.toml"),
    )
    .unwrap() // TODO Error handling
}
#[cfg(not(feature = "portable"))]
fn load_cache() -> AppCache {
    confy::load("plinks", "cache").unwrap() // TODO Error handling
}
#[cfg(feature = "portable")]
fn save_cache(cache: &AppCache) {
    confy::store_path(
        std::env::current_exe()
            .unwrap() // TODO Error handling
            .parent()
            .unwrap() // current_exe should never be root, unwrap is fine
            .join("config\\cache.toml"),
        cache,
    )
    .unwrap(); // TODO Error handling
}
#[cfg(not(feature = "portable"))]
fn save_cache(cache: &AppCache) {
    confy::store("plinks", "cache", cache).unwrap(); // TODO Error handling
}
