use configparser::ini::Ini;

#[derive(Debug)]
pub struct AppState {
    pub nginx_conf_path: String,
    pub nginx_conf_backup_path: String,
    pub enclave_image_initial_used_capacity_mb: u64,
    pub allotment_per_workerd_mb: u64,
    pub port: u16,
}

pub async fn get_config() -> AppState {
    let mut config = Ini::new();
    config.load("config.ini").unwrap();

    AppState {
        nginx_conf_path: config.get("env", "NGINX_CONF_PATH").unwrap(),
        nginx_conf_backup_path: config.get("env", "NGINX_CONF_BACKUP_PATH").unwrap(),
        enclave_image_initial_used_capacity_mb: config
            .get("env", "ENCLAVE_IMAGE_INITIAL_USED_CAPACITY_MB")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        allotment_per_workerd_mb: config
            .get("env", "ALLOTMENT_PER_WORKERD_MB")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        port: config.get("env", "PORT").unwrap().parse::<u16>().unwrap(),
    }
}
