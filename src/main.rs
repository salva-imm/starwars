use actix_web::{guard, web, web::Data, App, HttpResponse, HttpServer, Result};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use bb8_redis::{
    bb8,
    RedisConnectionManager
};
mod schema;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use crate::schema::models::{QueryRoot, StarWars, StarWarsSchema};

async fn index(schema: web::Data<StarWarsSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
    let source = playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = RedisConnectionManager::new("redis://localhost:6379").unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    // crash on startup if redis not available
    // let pool = pool.clone();
    // let mut conn = pool.get().await.unwrap();


    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(StarWars::new())
        .data(pool.clone())
        .finish();

    println!("Playground: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}