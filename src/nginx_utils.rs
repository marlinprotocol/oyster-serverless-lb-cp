use anyhow::Result;
use file_lock::{FileLock, FileOptions};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    process::Command,
};

pub const NGINX_CONF_PATH: &str = "/etc/nginx/nginx.conf";
pub const NGINX_CONF_BACKUP_PATH: &str = "/etc/nginx/nginx.conf.backup";
pub const ENCLAVE_IMAGE_INITIAL_USED_CAPACITY_MB: u64 = 2 * 1024;
pub const ALLOTMENT_PER_WORKERD_MB: u64 = 512;

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

pub async fn add_server(server: AddServerInfo) -> Result<(String, u64, u64)> {
    let ip = server.ip;
    let weight = server.capacity - ENCLAVE_IMAGE_INITIAL_USED_CAPACITY_MB;
    let max_conns = weight / ALLOTMENT_PER_WORKERD_MB;
    let line_to_add = format!("server {ip} weight={weight} max_conns={max_conns}");

    let options = FileOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true);

    let filelock = match FileLock::lock(NGINX_CONF_PATH, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting write lock: {:#?}", err),
    };

    let output = Command::new("cp")
        .arg(format!("{NGINX_CONF_PATH}"))
        .arg(format!("{NGINX_CONF_BACKUP_PATH}"))
        .output()
        .expect("Failed to create backup file");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create backup file: {}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    let file = File::open(NGINX_CONF_PATH).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut contents = String::new();

    let mut added: bool = false;
    while let Some(line) = lines.next() {
        let line = line.unwrap();

        if !added {
            if line.contains(format!("server {}", &ip).as_str()) {
                contents.push_str(format!("{line_to_add}").as_str());
                added = true;
            }
            if line.contains("# SERVERS -- END") {
                contents.push_str(format!("{line_to_add}\n").as_str());
                contents.push_str(&line);
                added = true;
            }
        } else {
            contents.push_str(&line);
        }
        contents.push_str("\n");
    }
    let mut file = File::create(NGINX_CONF_PATH).unwrap();
    file.write_all(contents.as_bytes()).unwrap();

    let res = soft_reload_nginx().await;
    if res.is_err() {
        let output = Command::new("mv")
            .arg(format!("{NGINX_CONF_BACKUP_PATH}"))
            .arg(format!("{NGINX_CONF_PATH}"))
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

    Ok((ip, weight, max_conns))
}

#[derive(Deserialize)]
pub struct RemoveServerInfo {
    pub ip: String,
}

pub async fn remove_server(ip: String) -> Result<bool> {
    let options = FileOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true);

    let filelock = match FileLock::lock(NGINX_CONF_PATH, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting write lock: {:#?}", err),
    };

    let output = Command::new("cp")
        .arg(format!("{NGINX_CONF_PATH}"))
        .arg(format!("{NGINX_CONF_BACKUP_PATH}"))
        .output()
        .expect("Failed to create backup file");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create backup file: {}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    let file = File::open(NGINX_CONF_PATH).unwrap();
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
    let mut file = File::create(NGINX_CONF_PATH).unwrap();
    file.write_all(contents.as_bytes()).unwrap();

    let res = soft_reload_nginx().await;
    if res.is_err() {
        let output = Command::new("mv")
            .arg(format!("{NGINX_CONF_BACKUP_PATH}"))
            .arg(format!("{NGINX_CONF_PATH}"))
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
