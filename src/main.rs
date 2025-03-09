use clap::Args;
use clap::command;
use yaml_rust2::Yaml;
use yaml_rust2::{YamlEmitter, YamlLoader};
// use clap::{App, Arg};
use dirs;
use regex::Regex;
// use serde_yaml;
use clap::Parser;
use std::collections::BTreeMap;
use clap::CommandFactory;
use std::f32::consts::TAU;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use walkdir::WalkDir;

fn get_config_path() -> std::path::PathBuf {
    let app_name = "rhy";
    let config_dir_path = dirs::config_dir().unwrap();
    return config_dir_path.join(app_name).join("config.yaml");
}

#[derive(Debug)]
struct Config {
    mount_path: String,
    cache_dir: String,
    remote_path: String,
}

enum ConfigKey {
    MountPath,
    CacheDir,
    RemotePath,
}

impl Config {
    fn read_config() -> Config {
        let config_path = get_config_path();
        if let Ok(mut file) = fs::File::open(&config_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let docs = YamlLoader::load_from_str(&contents).unwrap();

            let doc = &docs[0];

            return Config {
                mount_path: doc["mount_path"].as_str().unwrap_or("").to_string(),
                cache_dir: doc["cache_dir"].as_str().unwrap_or("").to_string(),
                remote_path: doc["remote_path"].as_str().unwrap_or("").to_string(),
            };
        } else {
            let init_conf = Config {
                mount_path: "/data/rcache".to_string(),
                cache_dir: "/remote".to_string(),
                remote_path: "vfs/".to_string(),
            };

            let parent = config_path.parent().unwrap();
            fs::create_dir_all(parent).unwrap();

            let mut map = Yaml::Hash(Default::default());

            if let Yaml::Hash(ref mut hash) = map {
                hash.insert(
                    Yaml::String("cache_dir".to_string()),
                    Yaml::String(init_conf.cache_dir.clone()),
                );
                hash.insert(
                    Yaml::String("mount_path".to_string()),
                    Yaml::String(init_conf.mount_path.clone()),
                );
                hash.insert(
                    Yaml::String("remote_path".to_string()),
                    Yaml::String(init_conf.remote_path.clone()),
                );
            }

            let mut out_str = String::new();
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&map).unwrap();

            let mut file = File::create(&config_path).unwrap();
            file.write_all(out_str.as_bytes()).unwrap();

            println!("YAML file created successfully!");
            return init_conf;
        }
    }

    fn set_config(mut self, config_key: ConfigKey, config: &str) {
        match config_key {
            ConfigKey::MountPath => {
                self.mount_path = config.to_string();
            }
            ConfigKey::CacheDir => {
                self.cache_dir = config.to_string();
            }
            ConfigKey::RemotePath => {
                self.remote_path = config.to_string();
            }
        }



        let config_path = get_config_path();
        let parent = config_path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();

        
        let mut map = Yaml::Hash(Default::default());
        
        if let Yaml::Hash(ref mut hash) = map {
            hash.insert(
                Yaml::String("cache_dir".to_string()),
                Yaml::String("/data/rcache".to_string()),
            );
            hash.insert(
                Yaml::String("mount_path".to_string()),
                Yaml::String("/remote".to_string()),
            );
            hash.insert(
                Yaml::String("remote_path".to_string()),
                Yaml::String("vfs/".to_string()),
            );
        }

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&map).unwrap();

        let mut file = File::create(&config_path).unwrap();
        file.write_all(out_str.as_bytes()).unwrap();
        
        println!("{} file created successfully!", config_path.display());
        println!("{:#?}", self);
    }

    fn get_cached_file_path(self, file: &PathBuf) -> PathBuf {
        let cache_dir = fs::canonicalize(std::path::Path::new(self.cache_dir.as_str()))
            .unwrap()
            .join(self.remote_path);
        let maped_file = map_cache_file(
            file,
            &fs::canonicalize(std::path::Path::new(self.mount_path.as_str())).unwrap(),
            &cache_dir,
        );

        return maped_file;
    }
}

fn get_file_updated_time(file: File) -> Option<SystemTime> {
    match file.metadata() {
        Ok(metadata) => {
            let modified = metadata.modified().unwrap();
            return Some(modified);
        }
        Err(e) => {
            panic!("Error accessing file metadata: {}", e);
        }
    }
}

fn get_file(file_path: &std::path::Path) -> File {
    match fs::File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Err {:?}", e);
            panic!("Error access file {:?}", file_path);
        }
    }
}

fn print_state(file_path: &std::path::Path) {
    // read file before access metadata
    let file = get_file(file_path);

    if let Some(modified) = get_file_updated_time(file) {
        let sys_time = SystemTime::now();
        let difference = sys_time.duration_since(modified);
        if let Ok(difference) = difference {
            println!("Updated before {:?}", difference);
        }
    }
}

fn remove_all_cache(config_map: &BTreeMap<String, String>) {
    let cache_path = std::path::Path::new(&config_map["cache_dir"]);
    if !cache_path.exists() {
        println!("Cache not exists {:?}", config_map["cache_dir"]);
        return;
    }
    if cache_path.is_dir() {
        for entry in WalkDir::new(cache_path) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                fs::remove_file(entry.path()).unwrap();
            }
        }
        println!("Cache removed: {:?}", cache_path);
    } else if cache_path.is_file() {
        fs::remove_file(cache_path).unwrap();
    }
}

fn map_cache_file(file: &PathBuf, mount_path: &PathBuf, cache_dir: &PathBuf) -> PathBuf {
    let parent = PathBuf::from(file.parent().unwrap());
    // println!("{:?} {:?}", parent, mount_path);
    if &parent == mount_path {
        return cache_dir.join(file.file_name().unwrap());
    } else {
        return map_cache_file(&parent, mount_path, cache_dir).join(file.file_name().unwrap());
    }
}

fn remove_cache_file(file_path: &PathBuf, verbose: bool) {
    if let Err(e) = fs::remove_file(file_path) {
        if verbose {
            println!("Cache not exists: {:?} \t {:?}", &file_path, e);
        }
    } else {
        if verbose {
            println!("Cache removed: {:?}", &file_path);
        }
    }
}

fn parse_duration_with_units(s: &str) -> Option<Duration> {
    let re = Regex::new(r"^(\d+)\s*(s|m|min|h)$").unwrap();
    if let Some(caps) = re.captures(s) {
        let num = caps.get(1).unwrap().as_str().parse::<u64>();
        let unit = caps.get(2).unwrap().as_str();

        if let Ok(num) = num {
            match unit {
                "s" => return Some(Duration::from_secs(num)),
                "m" | "min" => return Some(Duration::from_secs(num * 60)),
                "h" => return Some(Duration::from_secs(num * 60 * 60)),
                _ => panic!("Invalid unit"),
            }
        } else {
            panic!("Invalid number");
        }
    } else {
        panic!("Invalid format");
    }
}

use clap::Subcommand;

#[derive(Parser)]
#[command(name = "rhy <https://github.com/950288/rhy>")]
#[command(author = "95028 <950288s@gmail.com>")]
#[command(about = "A tool for track file state.", long_about = None)]
#[command(version, about, long_about = None)]
struct App {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    state: Option<String>,
    #[arg(short, long)]
    refresh: Option<String>,
    #[arg(short = 'a', long)]
    refresh_all: Option<String>,
    #[arg(short = 'T', long)]
    timeout: Option<u32>,
    #[arg(short = 't', long)]
    timeout_auto: bool,
}

#[derive(Subcommand)]
enum Commands {
    MountPath(Path),
    CacheDir(Path),
    RemotePath(Path),
    Info,
}

#[derive(Args)]
struct Path {
    name: String,
}

fn main() {
    let matches = App::parse();
    let config = Config::read_config();

    match &matches.command {
        Some(commands) => {
            match commands {
                Commands::MountPath(config_key) => {
                    config.set_config(ConfigKey::MountPath, &config_key.name);
                }
                Commands::CacheDir(config_key) => {
                    config.set_config(ConfigKey::CacheDir, &config_key.name);
                }
                Commands::RemotePath(config_key) => {
                    config.set_config(ConfigKey::RemotePath, &config_key.name);
                }
                Commands::Info => {
                    println!("Config path: {}", get_config_path().display());
                    println!("{:?}", config);
                }
            }
            return;
        }
        None => {}
    }

    match matches.state {
        Some(file) => {
            let file_path = std::path::Path::new(&file);
            print_state(&file_path);
            return;
        }
        None => {}
    }

    let timeout: u32 = match matches.timeout_auto {
        true => 20,
        false => match matches.timeout {
            Some(timeout) => timeout,
            None => 0,
        },
    };

    match matches.refresh {
        Some(file) => {
            let file_path = match fs::canonicalize(std::path::Path::new(&file)){
                Ok(file_path) => file_path,
                Err(e) => {
                    println!("File not exists: {:?}", &file);
                    panic!("Error: {:?}", e);
                }
            };
            let cached_file_path = config.get_cached_file_path(&file_path);
            remove_cache_file(&cached_file_path, true);
            print_state(&file_path);
            return;
        }
        None => {}
    }

    let mut cmd = App::command();
    cmd.print_help().unwrap();

    // let matches = App::new("")
    //     .version(env!("CARGO_PKG_VERSION"))
    //     .author("95028 <950288s@gmail.com>")
    //     .about("")
    //     .arg(
    //         Arg::with_name("config")
    //             .short("c")
    //             .long("config")
    //             .value_name("KEY VALUE")
    //             .help("Set config")
    //             .required(false)
    //             .number_of_values(2),
    //     )
    //     .arg(
    //         Arg::with_name("state")
    //             .short("s")
    //             .long("state")
    //             .value_name("FILE NAME")
    //             .help("Print state of file")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::with_name("refresh")
    //             .short("r")
    //             .long("refresh")
    //             .value_name("FILE")
    //             .help("Refresh file")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::with_name("refresh_all")
    //             .short("a")
    //             .long("refresh_all")
    //             .help("Refresh all files"),
    //     )
    //     .arg(
    //         Arg::with_name("timeout")
    //             .short("t")
    //             .long("timeout")
    //             .value_name("TIMEOUT")
    //             .help("Set timeout")
    //             .takes_value(true),
    //     )
    //     .get_matches();

    // let config_path = get_config_path();
    // let mut config_map = get_conf(config_path);

    // if let Some(values) = matches.values_of("config") {
    //     let values: Vec<&str> = values.collect();
    //     let key = values[0].to_string();
    //     let value = values[1].to_string();
    //     config_map = set_config(config_map, key, value);
    //     println!("Updated config: {:?}", config_map);
    //     return;
    // }

    // if let Some(file) = matches.value_of("state") {
    //     let file_path = fs::canonicalize(Path::new(&file)).unwrap();
    //     print_state(&file_path);
    //     return;
    // }

    // if let Some(file) = matches.value_of("refresh") {
    //     match fs::canonicalize(Path::new(&file)) {
    //         Ok(file_path) => {
    //             let cached_file_path = get_cached_file_path(&config_map, &file_path);
    //             if let Some(timeout) = matches.value_of("timeout") {
    //                 let timeout = parse_duration_with_units(timeout).unwrap();
    //                 print!(
    //                     "Detecting change of {:?} within past {:?}s .",
    //                     file,
    //                     timeout.as_secs()
    //                 );
    //                 io::stdout().flush().unwrap();
    //                 loop {
    //                     remove_cache_file(&cached_file_path, false);
    //                     let sys_time = SystemTime::now();
    //                     let updated_time = get_file_updated_time(&file_path).unwrap();
    //                     let difference = sys_time.duration_since(updated_time).unwrap();
    //                     if difference.as_secs() < timeout.as_secs() {
    //                         println!("\nUpdated before {:?}", difference);
    //                         break;
    //                     } else {
    //                         print!(".");
    //                         io::stdout().flush().unwrap();
    //                         std::thread::sleep(Duration::from_millis(200));
    //                     }
    //                 }
    //             } else {
    //                 remove_cache_file(&cached_file_path, true);
    //                 print_state(&file_path);
    //                 return;
    //             }
    //         },
    //         Err(e) => {
    //             panic!("Error: {:?}", e);
    //         }
    //     }
    //     return;
    // }

    // if matches.is_present("refresh_all") {
    //     remove_all_cache(&config_map);
    //     return;
    // }

    // let config_file = get_config_path();
    // println!(
    //     "Config file: {:?}",
    //     config_file.to_string_lossy().replace("\\", "/")
    // );
    // println!("{:?}", config_map);
}
