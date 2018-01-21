extern crate rocket_cors;

use data::*;
use fsm::*;
use rocket::*;

use rocket::*;
use handlebars::*;

use server_state::*;

use std::path::{Path, PathBuf};
use std::thread;
use std::sync::*;




pub fn spawn_web_server(data: InspectShared) {
    thread::spawn(move || {
        use rocket::config::{Config, Environment};
        use rocket::http::Method;
        use self::rocket_cors::{AllowedOrigins, AllowedHeaders};

        let config = Config::build(Environment::Staging)
            .address("0.0.0.0")
            .port(8002)
            .workers(1)
            .unwrap();


        //let (allowed_origins, failed_origins) = AllowedOrigins::some(&["http://localhost:4200"]);
        //assert!(failed_origins.is_empty());

        // You can also deserialize this
        let cors_options = self::rocket_cors::Cors {
            allowed_origins: AllowedOrigins::all(),
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
        };            
            
        //ignite()
        ::rocket::custom(config, true)
            .manage(data)
            //.attach(Template::fairing())
            .attach(cors_options)
            .mount("/", routes![index, fsm_info, viz_fsm_js])
            .launch();
    });
}

use rocket::response::content::Content;
use rocket::http::ContentType;

#[get("/")]
fn index(inspect: State<InspectShared>) -> Result<Content<String>, String> {
    let ctx = {
        let inspect = inspect.inner.lock().unwrap();
        PageIndex {
            fsm: inspect.machine_infos.clone(),
            name: "testis"
        }
    };

    let mut reg = Handlebars::new();
    
    match reg.template_render(include_str!("../templates/index.html.hbs"), &ctx) {
        Ok(s) => Ok(Content(ContentType::HTML, s)),
        Err(e) => Err(format!("Error rendering template: {:#?}", e))
    }
}

#[get("/fsm_info")]
fn fsm_info(inspect: State<InspectShared>) -> Content<String> {
    let inspect = inspect.inner.lock().unwrap();
    let data = &inspect.machine_infos;
    Content(ContentType::JSON, ::serde_json::to_string(&data).unwrap())
}

#[get("/viz_fsm.js")]
fn viz_fsm_js() -> Content<&'static str> {
    Content(ContentType::JavaScript, include_str!("../static/viz_fsm.js"))
}