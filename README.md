# actix-web-mongo-elastic-example

An actix-web example producing and consuming data from MongoDB and Elasticsearch.

## Routes

Four routes are available:

- `GET /v1/health`: Status of the application and dependencies
- `POST /v1/foos/init/{count}`: Saves/Index `count` documents in Mongo/ES
- `GET /v1/foos/{fooId}`: Retrieve a document from Mongo
- `GET /v1/foos/search`: Search documents from ES. It takes a json body:

```json
{
    "text": "foo 19",
    "from": 10,
    "size": 20
}
```

> `from` and `size` are optional (0 and 10 by default).
