mod config_load;
mod nginx_utils;
mod utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use nginx_utils::{AddServerInfo, RemoveServerInfo};

use crate::config_load::AppState;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/soft-reload")]
async fn soft_reload() -> impl Responder {
    let res = nginx_utils::soft_reload_nginx().await;

    if res.is_err() {
        return HttpResponse::InternalServerError().body(format!(
            "Failed to soft reload nginx. Error :{:?}",
            res.err().unwrap()
        ));
    }

    HttpResponse::Ok().body("Soft reload succesful")
}

#[post("/add-server")]
async fn add_server(
    web::Json(server): web::Json<AddServerInfo>,
    config: web::Data<AppState>,
) -> impl Responder {
    let res = nginx_utils::add_server(server, config).await;

    if res.is_err() {
        return HttpResponse::InternalServerError().body(format!(
            "Failed to add server. Error :{:?}",
            res.err().unwrap()
        ));
    }

    let res = res.unwrap();

    HttpResponse::Ok().body(format!(
        "Server ip: {} added successfully with weight: {} and max_conns: {}",
        res.0, res.1, res.2
    ))
}

#[post("/remove-server")]
async fn remove_server(
    web::Json(server): web::Json<RemoveServerInfo>,
    config: web::Data<AppState>,
) -> impl Responder {
    let res = nginx_utils::remove_server(server.ip, config).await;

    if res.is_err() {
        return HttpResponse::InternalServerError().body(format!(
            "Failed to remove server. Error :{:?}",
            res.err().unwrap()
        ));
    }

    let res = res.unwrap();

    if !res {
        return HttpResponse::BadRequest().body("Server with the IP is not registered");
    }

    HttpResponse::Ok().body("Server removed successfully")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config_load::get_config().await;
    let app_data = web::Data::new(AppState { ..config });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(index)
            .service(soft_reload)
            .service(add_server)
            .service(remove_server)
    })
    .bind(("0.0.0.0", config.port))
    .unwrap_or_else(|_| panic!("Can not bind to {}", &config.port))
    .run();

    println!("Server started on port {}", config.port);

    server.await
}
