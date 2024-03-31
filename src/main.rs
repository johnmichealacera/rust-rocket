use axum::{
    routing::get, routing::post, Router,
    extract::Json, body::Body, http::{self, Response},
    response::IntoResponse
};
use mongodb::{Client, options::ClientOptions, Database, error::Error};
use dotenv::dotenv;
use tokio::net::TcpListener;
use std::{
    env, sync::{Arc, Mutex}
};
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // Load the .env file
    dotenv().ok();
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/introduction", post(create_introduction))
        .route("/introductions", get(get_introductions));
    let axum_address = env::var("AXUM_ADDRESS").expect("AXUM_ADDRESS must be set");
    let app_port = env::var("PORT").expect("PORT must be set");
    let axum_listener_address = format!("{}:{}", axum_address, app_port);
    let listener = TcpListener::bind(&axum_listener_address).await.expect("Failed to bind to address");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct Introduction {
    title: String,
    icon: String,
}

fn return_data(data: String) -> Response<Body> {
    axum::http::Response::builder()
        .status(http::StatusCode::OK)
        .body(axum::body::Body::from(data))
        .unwrap()
}

fn return_error(err: Error) -> Response<Body>{
    eprintln!("Error finding documents: {}", err);
    axum::http::Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body("Error retrieving data".into())
        .unwrap()
}

async fn create_introduction(Json(payload): Json<Introduction>) -> Response<Body> {
    match connect_to_database().await {
        Ok(db) => {
            match insert_introductions(&db, "introductions", payload).await {
                Ok(()) => {
                    return_data(String::from("Inserted successfully"))
                }
                Err(err) => {
                    return_error(err)
                }
            }
        }
        Err(e) => {
            return_error(e)
        }
    }
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, JM AAcera man!"
}

async fn connect_to_database() -> Result<Database, mongodb::error::Error> {
    // Create a new MongoConnection instance
    let connection_result = MongoConnection::new().await;
    match connection_result {
        Ok(connection) => {
            // Connection successful
            println!("Connected to MongoDB");
            // Example usage: Get a handle to a database
            let db = connection.db("personal");
            Ok(db)
        }
        Err(e) => {
            // Handle connection error
            eprintln!("Error connecting to MongoDB: {}", e);
            Err(e)
        }
    }
}

async fn get_introductions() -> impl IntoResponse {
    match connect_to_database().await {
        Ok(database) => {
            match find_all(&database, "introductions").await {
                Ok(data) => {
                    return_data(data)
                }
                Err(err) => {
                    return_error(err)
                }
            }
        }
        Err(e) => {
            return_error(e)
        }
    }
}

async fn insert_introductions(db: &Database, collection_name: &str, data: Introduction) -> Result<(), mongodb::error::Error> {
    let collection: mongodb::Collection<Introduction> = db.collection(collection_name);
    collection.insert_one(data, None).await?;
    Ok(())
}

async fn find_all(db: &Database, collection_name: &str) -> Result<String, mongodb::error::Error> {
    let collection: mongodb::Collection<Value> = db.collection(collection_name);
    let mut cursor = collection.find(None, None).await?;
    let mut documents = Vec::new();
    while let Ok(result) = cursor.advance().await {
        match result {
            true => {
                match cursor.deserialize_current() {
                    Ok(doc) => { documents.push(doc); },
                    Err(e) => eprintln!("{:?}", e)
                }
            }
            false => break
        }
    }
    // Process the data obtained from find_all
    // Serialize the vector as JSON
    let json_data = serde_json::to_string(&documents).unwrap();
    Ok(json_data)
}


pub struct MongoConnection {
    client: Arc<Mutex<Client>>,
}

impl MongoConnection {
    pub async fn new() -> Result<Self, Error> {
        let mongo_db_uri = env::var("MONGO_DB_URI")
            .unwrap_or_else(|_| {
                println!("MONGO_DB_URI is not set, using default value");
                "default_value".to_string()
            });
        let client_options = ClientOptions::parse(mongo_db_uri).await?;
        let client = Client::with_options(client_options)?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub fn db(&self, name: &str) -> Database {
        self.client.lock().unwrap().database(name)
    }
}