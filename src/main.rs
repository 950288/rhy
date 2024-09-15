use clap::{App, Arg};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use dirs;
use std::io;
use std::fs;
use serde_yaml;
use regex::Regex;
use walkdir::WalkDir;
use std::time::Duration;
use std::time::SystemTime;
use std::collections::BTreeMap;

fn get_config_path() -> std::path::PathBuf {
    let app_name = "rhy";
    let config_dir_path = dirs::config_dir().unwrap();
    return config_dir_path.join(app_name).join("config.yaml");
}

fn set_config(mut config: BTreeMap<String, String>, key: String, value: String) -> BTreeMap<String, String> {
    config.insert(key, value);
    let yaml = serde_yaml::to_string(&config).unwrap();
    let config_path = get_config_path();
    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    return config;
}

fn get_conf(config_path: std::path::PathBuf) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    let file = fs::File::open(&config_path);
    if let Ok(file) = file {

        let reader = std::io::BufReader::new(file);

        let config: BTreeMap<String, String> = serde_yaml::from_reader(reader).unwrap();

        for (key, value) in config {
            if ["cache_dir", "remote_path", "mount_path"].contains(&key.as_str()) {
                map.insert(key, value);
            }
        }

        return map;
    } else {

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();

        map.insert("cache_dir".to_string(), "/data/rcache".to_string());
        map.insert("mount_path".to_string(), "/remote".to_string());
        map.insert("remote_path".to_string(), "vfs/".to_string());

        let yaml = serde_yaml::to_string(&map).unwrap();

        let mut file = fs::File::create(&config_path).unwrap();

        file.write_all(yaml.as_bytes()).unwrap();

        return map;
    }
}

fn get_file_updated_time(file_path: &PathBuf) -> Option<SystemTime> {
    let file = fs::File::open(file_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let _ = reader.bytes().count();

    match fs::metadata(file_path) {
        Ok(metadata) => {
            let modified = metadata.modified().unwrap();
            return Some(modified);
        },
        Err(e) => {
            panic!("Error accessing file metadata: {}", e);
        }  
    }
}

fn print_state(file_path: &PathBuf) {    
    // read file before access metadata
    let file = fs::File::open(file_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let _ = reader.bytes().count();

    if let Some(modified) = get_file_updated_time(file_path) {
        let sys_time = SystemTime::now();
        let difference = sys_time.duration_since(modified);
        if let Ok(difference) = difference {
            println!("Updated before {:?}", difference);
        }
    }
}

fn remove_all_cache(config_map: &BTreeMap<String, String>) {
    let cache_path = Path::new(&config_map["cache_dir"]);
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

fn get_cached_file_path(config_map: &BTreeMap<String, String>, file: &PathBuf) -> PathBuf {
    let cache_dir = fs::canonicalize(Path::new(&config_map["cache_dir"])).unwrap().join(&config_map["remote_path"]);
    let maped_file = map_cache_file(file, &fs::canonicalize(Path::new(&config_map["mount_path"])).unwrap(), &cache_dir);

    return maped_file;
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

fn main() {
    let matches = App::new("rhy <https://github.com/950288/rhy>")
        .version(env!("CARGO_PKG_VERSION"))
        .author("95028 <950288s@gmail.com>")
        .about("A tool for track file state.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("KEY VALUE")
                .help("Set config")
                .required(false)
                .number_of_values(2)
        )
        .arg(
            Arg::with_name("state")
                .short("s")
                .long("state")
                .value_name("FILE NAME")
                .help("Print state of file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("refresh")
                .short("r")
                .long("refresh")
                .value_name("FILE")
                .help("Refresh file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("refresh_all")
                .short("a")
                .long("refresh_all")
                .help("Refresh all files"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("TIMEOUT")
                .help("Set timeout")
                .takes_value(true),
        )
        .get_matches();

    let config_path = get_config_path();
    let mut config_map = get_conf(config_path);

    if let Some(values) = matches.values_of("config") {
        let values: Vec<&str> = values.collect();
        let key = values[0].to_string();
        let value = values[1].to_string();
        config_map = set_config(config_map, key, value);
        println!("Updated config: {:?}", config_map);
        return;
    }

    if let Some(file) = matches.value_of("state") {
        let file_path = fs::canonicalize(Path::new(&file)).unwrap();
        print_state(&file_path);
        return;
    }

    if let Some(file) = matches.value_of("refresh") {
        if let Ok(file_path) = fs::canonicalize(Path::new(&file)) {
            let cached_file_path =  get_cached_file_path(&config_map, &file_path);
            if let Some(timeout) = matches.value_of("timeout") {
                let timeout = parse_duration_with_units(timeout).unwrap();
                print!("Detecting change of {:?} within past {:?}s .", file, timeout.as_secs());
                io::stdout().flush().unwrap();
                loop {
                    remove_cache_file(&cached_file_path, false);
                    let sys_time = SystemTime::now();
                    let updated_time = get_file_updated_time(&file_path).unwrap();
                    let difference = sys_time.duration_since(updated_time).unwrap();
                    if difference.as_secs() < timeout.as_secs() {
                        println!("\nUpdated before {:?}", difference);
                        break;
                    } else {
                        print!(".");
                        io::stdout().flush().unwrap();
                        std::thread::sleep(Duration::from_millis(200));
                    }
                    
                }
            } else {
                remove_cache_file(&cached_file_path, true);
                print_state(&file_path);
                return;
            }
        } else {
            panic!("{:?} not exists", file);
        }
        return;
    }

    if matches.is_present("refresh_all") {
        remove_all_cache(&config_map);
        return;
    }

    
    let config_file = get_config_path();
    println!("Config file: {:?}", config_file.to_string_lossy().replace("\\", "/"));
    println!("{:?}", config_map);
}
