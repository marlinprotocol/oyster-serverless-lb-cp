use actix_web::web;
use anyhow::Result;
use file_lock::{FileLock, FileOptions};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    process::Command,
};

use crate::config_load::AppState;
use crate::utils::is_valid_ip_with_port;

pub async fn soft_reload_nginx() -> Result<()> {
    let output = Command::new("nginx")
        .arg("-s")
        .arg("reload")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to reload nginx: {}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct AddServerInfo {
    ip: String,
    capacity: u64,
}

pub async fn add_server(
    server: AddServerInfo,
    config: web::Data<AppState>,
) -> Result<(String, u64, u64)> {
    let ip = server.ip;
    if !is_valid_ip_with_port(&ip) {
        return Err(anyhow::anyhow!("Invalid IP address"));
    }
    if server.capacity <= config.enclave_image_initial_used_capacity_mb {
        return Err(anyhow::anyhow!(
            "Capacity must be greater than {}",
            config.enclave_image_initial_used_capacity_mb
        ));
    }
    let weight = server.capacity - config.enclave_image_initial_used_capacity_mb;
    let max_conns = weight / config.allotment_per_workerd_mb;
    let line_to_add = format!("server {ip} weight={weight} max_conns={max_conns}");

    let options = FileOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true);

    let filelock = match FileLock::lock(&config.nginx_conf_path, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting write lock: {:#?}", err),
    };

    let output = Command::new("cp")
        .arg(format!("{}", config.nginx_conf_path))
        .arg(format!("{}", config.nginx_conf_backup_path))
        .output()
        .expect("Failed to create backup file");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create backup file: {}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    let file = File::open(&config.nginx_conf_path).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut contents = String::new();

    let mut added: bool = false;
    while let Some(line) = lines.next() {
        let line = line.unwrap();

        if !added && line.contains(format!("server {}", &ip).as_str()) {
            contents.push_str(format!("{line_to_add}").as_str());
            added = true;
        } else if !added && line.contains("# SERVERS -- END") {
            contents.push_str(format!("{line_to_add}\n").as_str());
            contents.push_str(&line);
            added = true;
        } else {
            contents.push_str(&line);
        }
        contents.push_str("\n");
    }
    let mut file = File::create(&config.nginx_conf_path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();

    let res = soft_reload_nginx().await;
    if res.is_err() {
        let output = Command::new("mv")
            .arg(format!("{}", config.nginx_conf_backup_path))
            .arg(format!("{}", config.nginx_conf_path))
            .output()
            .expect("Failed to restore from backup file");

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to restore from backup file: {}",
                String::from_utf8(output.stderr).unwrap()
            ));
        }

        return Err(anyhow::anyhow!("Failed to add the ip to server and reload"));
    }

    let _ = filelock.unlock();

    Ok((ip, weight, max_conns))
}

#[derive(Deserialize)]
pub struct RemoveServerInfo {
    pub ip: String,
}

pub async fn remove_server(ip: String, config: web::Data<AppState>) -> Result<bool> {
    let options = FileOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true);

    let filelock = match FileLock::lock(&config.nginx_conf_path, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting write lock: {:#?}", err),
    };

    let output = Command::new("cp")
        .arg(format!("{}", config.nginx_conf_path))
        .arg(format!("{}", config.nginx_conf_backup_path))
        .output()
        .expect("Failed to create backup file");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create backup file: {}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    let file = File::open(&config.nginx_conf_path).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut contents = String::new();

    let mut removed = false;
    while let Some(line) = lines.next() {
        let line = line.unwrap();
        if line.contains(format!("server {}", &ip).as_str()) {
            removed = true;
        } else {
            contents.push_str(&line);
            contents.push_str("\n");
        }
    }
    let mut file = File::create(&config.nginx_conf_path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();

    let res = soft_reload_nginx().await;
    if res.is_err() {
        let output = Command::new("mv")
            .arg(format!("{}", config.nginx_conf_backup_path))
            .arg(format!("{}", config.nginx_conf_path))
            .output()
            .expect("Failed to restore from backup file");

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to restore from backup file: {}",
                String::from_utf8(output.stderr).unwrap()
            ));
        }
    }

    let _ = filelock.unlock();

    Ok(removed)
}
