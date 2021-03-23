mod configuration;
mod error;
mod handlers;
mod repository;

use std::error::Error;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use elasticsearch::http::transport::Transport;
use elasticsearch::Elasticsearch;
use env_logger::Env;
use mongodb::options::ClientOptions;
use mongodb::Client;

use configuration::Configuration;
use repository::{FooESRepo, FooMongoRepo};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let configuration = Configuration::load().await?;

    let mongodb_client = create_mongodb_client(&configuration).await?;
    let foo_mongo_repo = create_mongo_repo(&mongodb_client).await?;

    let es_client = create_es_client(&configuration).await?;
    let foo_es_repo = create_es_repo(&es_client).await?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(mongodb_client.clone())
            .data(es_client.clone())
            .data(foo_mongo_repo.clone())
            .data(foo_es_repo.clone())
            .service(
                web::scope("/v1")
                    .service(
                        web::scope("/foos")
                            .route("/init/{count}", web::post().to(handlers::init_data))
                            .route("/search", web::get().to(handlers::search_foo))
                            .route("/{fooId}", web::get().to(handlers::get_foo_by_id)),
                    )
                    .route("/health", web::get().to(handlers::health_check)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

async fn create_mongodb_client(
    configuration: &Configuration,
) -> Result<Client, mongodb::error::Error> {
    let mdb_client_options = ClientOptions::parse(&configuration.mongodb_connection_string).await?;
    Client::with_options(mdb_client_options)
}

async fn create_mongo_repo(mongodb_client: &Client) -> Result<FooMongoRepo, mongodb::error::Error> {
    Ok(FooMongoRepo::new(mongodb_client))
}

async fn create_es_client(
    configuration: &Configuration,
) -> Result<Elasticsearch, elasticsearch::Error> {
    let es_transport = Transport::single_node(&configuration.elasticsearch_url)?;
    Ok(Elasticsearch::new(es_transport))
}

async fn create_es_repo(es_client: &Elasticsearch) -> Result<FooESRepo, elasticsearch::Error> {
    Ok(FooESRepo::new(es_client.clone()))
}
