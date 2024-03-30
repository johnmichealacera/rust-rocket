use axum::{
    routing::get, Router
};
use mongodb::{Client, options::ClientOptions, Database, error::Error};
use dotenv::dotenv;
use std::env;
use std::sync::{Arc, Mutex};
use serde_json::Value;

#[tokio::main]
async fn main() {
    // Load the .env file
    dotenv().ok();
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/jm", get(get_jm));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, JM AAcera man!"
}

async fn get_jm() -> &'static str {
    // Create a new MongoConnection instance
    let connection_result = MongoConnection::new().await;
    let db;
    match connection_result {
        Ok(connection) => {
            // Connection successful, use the connection object here
            println!("Connected to MongoDB");
            // Example usage: Get a handle to a database
            db = connection.db("personal");
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
            if let Err(err) = find_all(&db, "introductions").await {
                eprintln!("Error finding documents: {}", err);
            }
        }
        Err(e) => eprintln!("Error connecting to MongoDB: {}", e),
    }
    "Hello, JM is a new endpoint"
}

async fn find_all(db: &Database, collection_name: &str) -> Result<(), mongodb::error::Error> {
    let collection: mongodb::Collection<Value> = db.collection(collection_name);
    let mut cursor = collection.find(None, None).await?;
    while let Ok(result) = cursor.advance().await {
        println!("{}", result);
        match result {
            true => {
                match cursor.deserialize_current() {
                    Ok(doc) => println!("{:?}", doc),
                    Err(e) => eprintln!("{:?}", e)
                }
            }
            false => break
        }
    }

    println!("this is the end of loop");
    // Process the cursor...
    Ok(())
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