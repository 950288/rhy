use clap::{App, Arg};
use dirs;
use serde_yaml;
use std::collections::BTreeMap;
use std::io::Write;
use std::env;
use std::path::Path;
use std::fs;
use std::time::SystemTime;

fn get_config_path() -> std::path::PathBuf {
    let app_name = "rhy";
    let config_dir_path = dirs::config_dir().unwrap();
    return config_dir_path.join(app_name).join("config.yaml");
}

fn set_config(mut config: BTreeMap<String, String>, key: String, value: String) -> BTreeMap<String, String> {
    config.insert(key, value);
    let yaml = serde_yaml::to_string(&config).unwrap();
    let config_path = get_config_path();
    let mut file = std::fs::File::create(&config_path).unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    return config;
}

fn get_conf(config_path: std::path::PathBuf) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    let file = std::fs::File::open(&config_path);
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

        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();

        map.insert("cache_dir".to_string(), "/data/rcache".to_string());
        map.insert("remote_path".to_string(), "/dev/".to_string());
        map.insert("mount_path".to_string(), "/remote".to_string());

        let yaml = serde_yaml::to_string(&map).unwrap();

        let mut file = std::fs::File::create(&config_path).unwrap();

        file.write_all(yaml.as_bytes()).unwrap();

        return map;
    }
}

fn main() {


    let matches = App::new("rhy")
        .version("0.1.0")
        .author("Rhy")
        .about("A tool for track file state")
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
        println!("{:?}", config_map);
        return;
    }

    if let Some(file) = matches.value_of("state") {
        let file_path = fs::canonicalize(Path::new(&file)).unwrap();
        
        // print!("{:?}", file_path);

        match fs::metadata(file_path) {
            Ok(metadata) => {
                let modified = metadata.modified().unwrap();
                
                let sys_time = SystemTime::now();

                let difference = sys_time.duration_since(modified);
                if let Ok(difference) = difference {
                    println!("Updated: {:?}", difference);
                }
            },
            Err(e) => {
                println!("Error accessing file metadata: {}", e);
            }
        }


        return;
    }

    if matches.is_present("refresh_all") {
        println!("{:?}", config_map);
        return;
    }

    if let Some(file) = matches.value_of("refresh") {
        println!("{:?}", file);
        return;
    }

    if let Some(timeout) = matches.value_of("timeout") {
        println!("{:?}", timeout);
        return;
    }
    
    println!("{:?}", config_map);
}
