use rocket::{get, routes};

#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/jm")]
fn helloJM() -> String {
    format!("Hello JM my friend, ")
}

#[rocket::main]
async fn main() {
   let _ = rocket::build().mount("/hello", routes![hello, helloJM]).launch().await;
}
