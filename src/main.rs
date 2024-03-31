use axum::http::{self};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{
    routing::get, Router,
    extract::Json,
};
use mongodb::{Client, options::ClientOptions, Database, error::Error};
use dotenv::dotenv;
use tokio::net::TcpListener;
use std::env;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use axum::extract::Path;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // Load the .env file
    dotenv().ok();
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/user/:users", get(root_param))
        .route("/json", post(create_introduction))
        // .route("/json", post(insert_introductions))
        .route("/jm", get(get_jm));
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

async fn create_introduction(Json(payload): Json<Introduction>) -> impl IntoResponse {
    println!("I am here");
    println!("{:?}", payload);
    // Create a new MongoConnection instance
    let connection_result = MongoConnection::new().await;
    match connection_result {
        Ok(connection) => {
            // Connection successful, use the connection object here
            println!("Connected to MongoDB");
            // Example usage: Get a handle to a database
            let db = connection.db("personal");
            // Retrieve all collection names in the database
            match db.list_collection_names(None).await {
                Ok(collections) => {
                    println!("Available collections:");
                    for collection_name in collections {
                        println!("{}", collection_name);
                    }
                }
                Err(err) => eprintln!("Error listing collections: {}", err),
            }
            match insert_introductions(&db, "introductions", payload).await {
                Ok(()) => {
                    axum::http::Response::builder()
                        .status(http::StatusCode::OK)
                        .body(axum::body::Body::from("Inserted successfully"))
                        .unwrap()
                }
                Err(err) => {
                    // Handle error from find_all
                    eprintln!("Error finding documents: {}", err);
                    // Return an error response
                    axum::http::Response::builder()
                        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error retrieving data".into())
                        .unwrap()
                }
            }
        }
        Err(e) => {
            // Handle connection error
            eprintln!("Error connecting to MongoDB: {}", e);
            // Return an error response
            axum::http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error connecting to MongoDB".into())
                .unwrap()
        }
    }
}

// basic handler that responds with a static string
async fn root_param(Path(id): Path<u64>) -> String {
    println!("{}", id);
    format!("Hello, JM AAcera man param with ID: {}", id)
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, JM AAcera man!"
}


async fn get_jm() -> impl IntoResponse {
    // Create a new MongoConnection instance
    let connection_result = MongoConnection::new().await;
    match connection_result {
        Ok(connection) => {
            // Connection successful, use the connection object here
            println!("Connected to MongoDB");
            // Example usage: Get a handle to a database
            let db = connection.db("personal");
            // Retrieve all collection names in the database
            match db.list_collection_names(None).await {
                Ok(collections) => {
                    println!("Available collections:");
                    for collection_name in collections {
                        println!("{}", collection_name);
                    }
                }
                Err(err) => eprintln!("Error listing collections: {}", err),
            }
            match find_all(&db, "introductions").await {
                Ok(data) => {
                    // Process the data obtained from find_all
                    // For demonstration, returning a success response with a message
                    // Serialize the vector as JSON
                    let json_data = serde_json::to_string(&data).unwrap();
                    axum::http::Response::builder()
                        .status(http::StatusCode::OK)
                        .body(axum::body::Body::from(json_data))
                        .unwrap()
                }
                Err(err) => {
                    // Handle error from find_all
                    eprintln!("Error finding documents: {}", err);
                    // Return an error response
                    axum::http::Response::builder()
                        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error retrieving data".into())
                        .unwrap()
                }
            }
        }
        Err(e) => {
            // Handle connection error
            eprintln!("Error connecting to MongoDB: {}", e);
            // Return an error response
            axum::http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error connecting to MongoDB".into())
                .unwrap()
        }
    }
}

async fn insert_introductions(db: &Database, collection_name: &str, data: Introduction) -> Result<(), mongodb::error::Error> {
    let collection: mongodb::Collection<Introduction> = db.collection(collection_name);
    collection.insert_one(data, None).await?;
    Ok(())
}

async fn find_all(db: &Database, collection_name: &str) -> Result<Vec<Value>, mongodb::error::Error> {
    let collection: mongodb::Collection<Value> = db.collection(collection_name);
    let mut cursor = collection.find(None, None).await?;
    let mut documents = Vec::new();
    while let Ok(result) = cursor.advance().await {
        println!("{}", result);
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

    println!("{:?}", documents);

    println!("this is the end of loop");
    // Process the cursor...
    Ok(documents)
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