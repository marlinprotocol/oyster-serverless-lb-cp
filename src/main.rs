mod nginx_utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use nginx_utils::{AddServerInfo, RemoveServerInfo};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/soft-reload")]
async fn soft_reload() -> impl Responder {
    let res = nginx_utils::soft_reload_nginx().await;

    if !res.is_err() {
        return HttpResponse::InternalServerError().body(format!(
            "Failed to soft reload nginx. Error :{:?}",
            res.err().unwrap()
        ));
    }

    HttpResponse::Ok().body("Soft reload succesful")
}

#[post("/add-server")]
async fn add_server(web::Json(server): web::Json<AddServerInfo>) -> impl Responder {
    let res = nginx_utils::add_server(server).await;

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
async fn remove_server(web::Json(server): web::Json<RemoveServerInfo>) -> impl Responder {
    let res = nginx_utils::remove_server(server.ip).await;

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
    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(soft_reload)
            .service(add_server)
            .service(remove_server)
    })
    .bind(("000000000", 8012))?
    .run()
    .await
}
