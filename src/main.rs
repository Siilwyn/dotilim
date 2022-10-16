#[macro_use]
extern crate crossbeam;
use glob::glob;
use rand::seq::IteratorRandom;
use serde::Deserialize;

use std::fs;
use std::path;
use std::process::Command;
use std::time::Duration;

#[derive(Deserialize, Debug)]
#[allow(non_camel_case_types)]
enum Order {
    random,
    alphabetical,
}

#[derive(Deserialize, Debug)]
struct Config {
    #[allow(unused)]
    version: u64,
    sources: Option<Vec<String>>,
    sources_light: Option<Vec<String>>,
    sources_dark: Option<Vec<String>>,
    duration: u64,
    order: Order,
}

fn main() {
    let config = dirs::config_dir()
        .map(|config_dir| read_config(config_dir.join("dotilim.toml")))
        .unwrap();

    let wallpapers_default = expand_sources(config.sources);
    let wallpapers_light = expand_sources(config.sources_light);
    let wallpapers_dark = expand_sources(config.sources_dark);

    let ticker = crossbeam::channel::tick(Duration::from_secs(config.duration));

    loop {
        select! {
            recv(ticker) -> _ => {
                let color_scheme = get_color_scheme().unwrap();
                let wallpapers = match color_scheme {
                    ColorScheme::NoPreference | ColorScheme::PreferLight => [wallpapers_default.as_slice(), wallpapers_light.as_slice()].concat(),
                    ColorScheme::PreferDark => [wallpapers_default.as_slice(), wallpapers_dark.as_slice()].concat(),
                };

                match config.order {
                    Order::random => {
                        change_wallpaper(
                            &pick_random(&wallpapers).into_os_string().into_string().unwrap()
                        )
                    }
                    Order::alphabetical => {
                        change_wallpaper(
                            &wallpapers.get(0).unwrap().clone().into_os_string().into_string().unwrap()
                        )
                    }
                }
            },
        };
    }
}

fn change_wallpaper(background_uri: &str) {
    Command::new("gsettings")
        .args(&[
            "set",
            "org.gnome.desktop.background",
            "picture-uri",
            background_uri,
        ])
        .status()
        .expect("Failed to change wallpaper using Gnome.");

    Command::new("gsettings")
        .args(&[
            "set",
            "org.gnome.desktop.background",
            "picture-uri-dark",
            background_uri,
        ])
        .status()
        .expect("Failed to change wallpaper using Gnome.");

    println!("Set wallpaper to {}", background_uri);
}

fn read_config(path: path::PathBuf) -> Config {
    let toml_str = fs::read_to_string(path).expect("No config file found");

    toml::from_str(&toml_str).unwrap()
}

fn expand_sources(sources: Option<Vec<String>>) -> Vec<path::PathBuf> {
    let paths: Vec<_> = sources
        .unwrap_or_default()
        .iter()
        .map(|source| shellexpand::full(source).unwrap())
        .flat_map(|source| glob(&source).unwrap().filter_map(Result::ok))
        .collect();

    paths
}

/// The system's preferred color scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorScheme {
    NoPreference,
    PreferDark,
    PreferLight,
}

fn get_color_scheme() -> Option<ColorScheme> {
    let connection = zbus::blocking::Connection::session();
    if connection.is_err() {
        return None;
    }

    let reply = connection.unwrap().call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.portal.Settings"),
        "Read",
        &("org.freedesktop.appearance", "color-scheme"),
    );

    if let Ok(reply) = &reply {
        let theme = reply.body::<zvariant::Value>();
        if theme.is_err() {
            return None;
        }
        let theme = theme.unwrap().downcast::<u32>();
        match theme.unwrap() {
            1 => Some(ColorScheme::PreferDark),
            2 => Some(ColorScheme::PreferLight),
            _ => Some(ColorScheme::NoPreference),
        }
    } else {
        None
    }
}

fn pick_random(paths: &[path::PathBuf]) -> path::PathBuf {
    let mut rng = rand::thread_rng();

    let random_item = paths
        .iter()
        .choose(&mut rng)
        .expect("sources should not be empty");

    random_item.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_config_simple() {
        assert_eq!(
            read_config(path::PathBuf::from("./test/config-simple.toml")).version,
            1
        );

        assert_eq!(
            read_config(path::PathBuf::from("./test/config-simple.toml"))
                .sources
                .unwrap(),
            [String::from("./sources-fixture/*.jpg")]
        );
    }

    #[test]
    fn expand_sources_globs() {
        assert_eq!(
            expand_sources(Some(vec![String::from("./test/sources-fixture/*.jpg")])),
            [
                path::PathBuf::from("test/sources-fixture/a.jpg"),
                path::PathBuf::from("test/sources-fixture/b.jpg")
            ]
        )
    }

    #[test]
    fn expand_sources_multiple_globs() {
        assert_eq!(
            expand_sources(Some(vec![
                String::from("./test/sources-fixture/*.jpg"),
                String::from("./test/sources-fixture/more/*.jpg")
            ])),
            [
                path::PathBuf::from("test/sources-fixture/a.jpg"),
                path::PathBuf::from("test/sources-fixture/b.jpg"),
                path::PathBuf::from("test/sources-fixture/more/a.jpg"),
                path::PathBuf::from("test/sources-fixture/more/c.jpg")
            ]
        )
    }

    #[test]
    fn expand_sources_empty() {
        assert!(expand_sources(None).is_empty())
    }

    #[test]
    fn pick_random_simple() {
        assert_eq!(
            pick_random([path::PathBuf::from("test/sources-fixture/a.jpg")].as_ref()),
            path::PathBuf::from("test/sources-fixture/a.jpg")
        )
    }
}
