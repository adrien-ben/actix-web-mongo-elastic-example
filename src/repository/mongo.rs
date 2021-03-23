use mongodb::bson::{self, doc};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Deserialize, Serialize, Debug)]
pub struct FooEntity {
    pub _id: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct FooMongoRepo {
    collection: Collection,
}

impl FooMongoRepo {
    const BD_NAME: &'static str = "test";
    const COLLECTION_NAME: &'static str = "foo";

    pub fn new(client: &Client) -> Self {
        Self {
            collection: client
                .database(FooMongoRepo::BD_NAME)
                .collection(FooMongoRepo::COLLECTION_NAME),
        }
    }

    pub async fn init(&self, count: usize) -> Result<(), AppError> {
        let mut foos = Vec::with_capacity(count);
        for i in 0..count {
            foos.push(bson::to_document(&FooEntity {
                _id: i.to_string(),
                description: format!("I am foo number {}", i),
            })?);
        }

        self.collection.delete_many(doc! {}, None).await?;
        self.collection.insert_many(foos, None).await?;

        Ok(())
    }

    pub async fn find_foo_by_id(&self, id: String) -> Result<Option<FooEntity>, AppError> {
        let entity = self
            .collection
            .find_one(doc! { "_id": &id }, None)
            .await?
            .map(bson::from_document::<FooEntity>)
            .transpose()?;
        Ok(entity)
    }
}
