use libocp::ocp_core::Ocp;
use actix_web::{middleware, web, HttpServer, App, HttpResponse};

async fn index() -> HttpResponse {
    let mut ocp = Ocp::connect().unwrap();

    let device_name = "ovey0".to_string();
    let parent_device_name = "rxe0".to_string();
    let network_uuid_str = "c929e96d-6285-4528-b98e-b364d64790ae".to_string();

    let node_guid_he = 0xdead_beef_0bad_f00d_u64;


    let exists = ocp.ocp_get_device_info(&device_name);
    if exists.is_ok() {
        let _ = ocp.ocp_delete_device(&device_name).unwrap();
    }

    let res = ocp.ocp_create_device(
        &device_name,
        &parent_device_name,
        node_guid_he,
        &network_uuid_str
    ).expect("Must be created!");

    let res = ocp.ocp_get_device_info(
        &device_name,
    );


    let res = res.unwrap();

    // HttpResponse::Ok().json(res)
    HttpResponse::Ok().json(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    println!("Starting REST service on localhost:{}", 12345);

    std::env::set_var("RUST_LOG", "actix_web=info,debug");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
    })
        .bind(format!("localhost:{}", 12345))?
        .run()
        .await
}
