use elasticsearch::{BulkOperation, BulkParts, Elasticsearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::error::AppError;

#[derive(Deserialize, Debug, Validate)]
pub struct FooSearchRequest {
    #[serde(default = "FooSearchRequest::default_from")]
    from: u32,
    #[serde(default = "FooSearchRequest::default_size")]
    #[validate(range(
        min = 1,
        max = 100,
        message = "size must be at least 1 and at most 100"
    ))]
    size: u32,
    #[validate(length(min = 1, message = "text must not be blank"))]
    text: String,
}

impl FooSearchRequest {
    fn default_from() -> u32 {
        0
    }

    fn default_size() -> u32 {
        10
    }
}

#[derive(Deserialize, Debug)]
struct SearchResponse<T> {
    hits: Hits<T>,
}

#[derive(Deserialize, Debug)]
struct Hits<T> {
    hits: Vec<Hit<T>>,
}

#[derive(Deserialize, Debug)]
struct Hit<T> {
    _source: T,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FooESEntity {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct FooESRepo {
    client: Elasticsearch,
}

impl FooESRepo {
    const INDEX_NAME: &'static str = "foo";

    pub fn new(client: Elasticsearch) -> Self {
        Self { client }
    }

    pub async fn init(&self, count: usize) -> Result<(), AppError> {
        let mut bulk_ops = Vec::<BulkOperation<_>>::with_capacity(count);
        for i in 0..count {
            let entity = FooESEntity {
                id: i.to_string(),
                description: format!("I am foo number {}", i),
            };
            bulk_ops.push(BulkOperation::index(entity).id(i.to_string()).into());
        }

        self.client
            .bulk(BulkParts::Index(FooESRepo::INDEX_NAME))
            .body(bulk_ops)
            .send()
            .await?;

        Ok(())
    }

    pub async fn search_foo_by_text(
        &self,
        request: FooSearchRequest,
    ) -> Result<Vec<FooESEntity>, AppError> {
        let body = json!({
            "query": {
                "match": {
                    "description": {
                        "query": &request.text
                    }
                }
            }
        });
        let search_response = self
            .client
            .search(SearchParts::Index(&[FooESRepo::INDEX_NAME]))
            .body(body)
            .from(request.from as _)
            .size(request.size as _)
            .send()
            .await?
            .json::<SearchResponse<FooESEntity>>()
            .await?;

        Ok(search_response
            .hits
            .hits
            .into_iter()
            .map(|hit| hit._source)
            .collect())
    }
}
