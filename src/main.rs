mod nginx_utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use nginx_utils::ServerInfo;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/soft-reload")]
async fn soft_reload() -> impl Responder {
    let res = nginx_utils::soft_reload_nginx().await;

    if !res.is_err() {
        return HttpResponse::InternalServerError().body("Failed to soft reload nginx");
    }

    HttpResponse::Ok().body("Soft reload succesful")
}

#[post("/add-server")]
async fn add_server(web::Json(server): web::Json<ServerInfo>) -> impl Responder {
    let res = nginx_utils::add_server(server).await;

    if res.is_err() {
        return HttpResponse::InternalServerError().body("Failed to add server");
    }

    let res = res.unwrap();

    HttpResponse::Ok().body(format!(
        "Server ip: {} added successfully with weight: {} and max_conns: {}",
        res.0, res.1, res.2
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(soft_reload)
            .service(add_server)
    })
    .bind(("000000000", 8012))?
    .run()
    .await
}
