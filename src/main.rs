#[macro_use]
extern crate crossbeam;
use crossbeam::crossbeam_channel;
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
    version: u64,
    sources: Vec<String>,
    duration: u64,
    order: Order,
}

fn main() {
    let config = dirs::config_dir()
        .map(|config_dir| read_config(config_dir.join("dotilim.toml")))
        .unwrap();

    let wallpapers = expand_sources(config.sources);

    let ticker = crossbeam_channel::tick(Duration::from_secs(config.duration));

    loop {
        select! {
            recv(ticker) -> _ => {
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
            &background_uri,
        ])
        .status()
        .expect("Failed to change wallpaper using Gnome.");

    println!("Set wallpaper to {}", background_uri);
}

fn read_config(path: path::PathBuf) -> Config {
    let toml_str = fs::read_to_string(path).expect("No config file found");

    toml::from_str(&toml_str).unwrap()
}

fn expand_sources(sources: Vec<String>) -> Vec<path::PathBuf> {
    let paths: Vec<_> = sources
        .iter()
        .map(|source| shellexpand::full(source).unwrap())
        .map(|source| glob(&source).unwrap().filter_map(Result::ok))
        .flatten()
        .collect();

    paths
}

fn pick_random(paths: &[path::PathBuf]) -> path::PathBuf {
    let mut rng = rand::thread_rng();

    let random_item = paths.iter().choose(&mut rng).unwrap();

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
            read_config(path::PathBuf::from("./test/config-simple.toml")).sources,
            [String::from("./sources-fixture/*.jpg")]
        );
    }

    #[test]
    fn expand_sources_globs() {
        assert_eq!(
            expand_sources(vec![String::from("./test/sources-fixture/*.jpg")]),
            [
                path::PathBuf::from("test/sources-fixture/a.jpg"),
                path::PathBuf::from("test/sources-fixture/b.jpg")
            ]
        )
    }

    #[test]
    fn expand_sources_multiple_globs() {
        assert_eq!(
            expand_sources(vec![
                String::from("./test/sources-fixture/*.jpg"),
                String::from("./test/sources-fixture/more/*.jpg")
            ]),
            [
                path::PathBuf::from("test/sources-fixture/a.jpg"),
                path::PathBuf::from("test/sources-fixture/b.jpg"),
                path::PathBuf::from("test/sources-fixture/more/a.jpg"),
                path::PathBuf::from("test/sources-fixture/more/c.jpg")
            ]
        )
    }

    #[test]
    fn pick_random_simple() {
        assert_eq!(
            pick_random(&[path::PathBuf::from("test/sources-fixture/a.jpg")].to_vec()),
            path::PathBuf::from("test/sources-fixture/a.jpg")
        )
    }
}
