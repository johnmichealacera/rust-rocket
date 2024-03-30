// use rocket::{get, routes};

// #[get("/<name>/<age>")]
// fn hello(name: &str, age: u8) -> String {
//     format!("Hello, {} year old named {}!", age, name)
// }

// #[get("/jm")]
// fn hello_jm() -> String {
//     format!("Hello JM my friend, ")
// }

// #[rocket::main]
// async fn main() {
//    let _ = rocket::build()
//     .mount("/hello", routes![hello, hello_jm])
//     .launch()
//     .await;
// }
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, JM!"
}

