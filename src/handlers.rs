use actix_web::{web, HttpResponse, Result};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::AppError;
use crate::repository::*;
use elasticsearch::cluster::ClusterHealthParts;
use elasticsearch::Elasticsearch;
use mongodb::Client;
use std::ops::BitAnd;

#[derive(Serialize, Debug)]
struct Foo {
    id: String,
    description: String,
}

impl From<FooEntity> for Foo {
    fn from(entity: FooEntity) -> Self {
        Foo {
            id: entity._id,
            description: entity.description,
        }
    }
}

impl From<FooESEntity> for Foo {
    fn from(entity: FooESEntity) -> Self {
        Foo {
            id: entity.id,
            description: entity.description,
        }
    }
}

pub async fn init_data(
    count: web::Path<usize>,
    foo_es_repo: web::Data<FooESRepo>,
    foo_mongo_repo: web::Data<FooMongoRepo>,
) -> Result<HttpResponse, AppError> {
    foo_es_repo.init(*count).await?;
    foo_mongo_repo.init(*count).await?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn search_foo(
    web::Json(search_request): web::Json<FooSearchRequest>,
    foo_es_repo: web::Data<FooESRepo>,
) -> Result<HttpResponse, AppError> {
    search_request.validate()?;

    let foos = foo_es_repo
        .search_foo_by_text(search_request)
        .await?
        .into_iter()
        .map(Foo::from)
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(foos))
}

pub async fn get_foo_by_id(
    foo_id: web::Path<String>,
    foo_mongo_repo: web::Data<FooMongoRepo>,
) -> Result<HttpResponse, AppError> {
    foo_mongo_repo
        .find_foo_by_id(foo_id.to_string())
        .await?
        .map(Foo::from)
        .map(|foo| HttpResponse::Ok().json(foo))
        .ok_or(AppError::NotFound)
}

#[derive(Serialize, Debug)]
struct Health {
    status: Status,
}

#[derive(Serialize, PartialEq, Debug)]
enum Status {
    UP,
    DOWN,
}

impl BitAnd for Status {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self == Self::UP && rhs == Self::UP {
            Status::UP
        } else {
            Status::DOWN
        }
    }
}

pub async fn health_check(
    mongodb_client: web::Data<Client>,
    es_client: web::Data<Elasticsearch>,
) -> HttpResponse {
    let mongo_status = get_mongo_status(&mongodb_client).await;
    let es_status = get_es_status(&es_client).await;

    let status = mongo_status & es_status;

    HttpResponse::Ok().json(Health { status })
}

async fn get_mongo_status(mongo_client: &Client) -> Status {
    mongo_client
        .list_database_names(None, None)
        .await
        .map_or(Status::DOWN, |_| Status::UP)
}

async fn get_es_status(es_client: &Elasticsearch) -> Status {
    #[derive(Deserialize, Debug)]
    struct HealthResponse {
        status: HealthStatus,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(rename_all = "lowercase")]
    enum HealthStatus {
        GREEN,
        YELLOW,
        RED,
    }

    let response = es_client
        .cluster()
        .health(ClusterHealthParts::None)
        .send()
        .await;

    if response.is_err() {
        return Status::DOWN;
    }

    response
        .unwrap()
        .json::<HealthResponse>()
        .await
        .map_or(Status::DOWN, |h| {
            if h.status == HealthStatus::RED {
                Status::DOWN
            } else {
                Status::UP
            }
        })
}
