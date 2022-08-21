

use async_graphql::{
    connection::{query, Connection, Edge},
    Context, Enum, Error, Interface, Object, OutputType, Result,
};
use serde::{Deserialize, Serialize};

use async_graphql::{EmptyMutation, EmptySubscription, Schema};

pub type StarWarsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

use bb8_redis::{
    bb8,
    redis::cmd,
    RedisConnectionManager,
};
use bb8_redis::bb8::PooledConnection;
use futures::future;


/// One of the films in the Star Wars Trilogy
#[derive(Enum, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum Episode {
    /// Released in 1977.
    NewHope,

    /// Released in 1980.
    Empire,

    /// Released in 1983.
    Jedi,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StarWarsChar {
    id: String,
    name: String,
    is_human: bool,
    friends: Vec<usize>,
    appears_in: Vec<Episode>,
    home_planet: Option<String>,
    primary_function: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Human(StarWarsChar);

async fn get_redis_conn<'ctx>(ctx: &Context<'ctx>) -> PooledConnection<'ctx, RedisConnectionManager>{
    let redis_client = ctx.data_unchecked::<bb8::Pool<RedisConnectionManager>>();
    redis_client.get().await.unwrap()
}

async fn get_data_from_redis(conn: &mut bb8_redis::redis::aio::Connection, key: String) -> Result<String> {
    let key: Vec<String> = cmd("KEYS")
        .arg(key)
        .query_async(conn)
        .await.unwrap();
    let data: String = cmd("GET")
        .arg(key.get(0))
        .query_async(conn)
        .await?;
    Ok(data)
}

/// A humanoid creature in the Star Wars universe.
#[Object]
impl Human {
    /// The id of the human.
    async fn id(&self) -> String {
        self.0.id.to_string()
    }

    /// The name of the human.
    async fn name(&self) -> String {
        self.0.name.to_string()
    }

    /// The friends of the human, or an empty list if they have none.
    async fn friends<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<Character> {
        let redis_client = ctx.data_unchecked::<bb8::Pool<RedisConnectionManager>>();
        let x = self.0.friends.iter().map(|id| async move {
            let mut conn = redis_client.get().await.unwrap();
            let reply: String = get_data_from_redis(&mut *conn, format!("{}*", id)).await.unwrap();
            let friend: StarWarsChar = serde_json::from_str(&reply).unwrap();
            if friend.is_human{
                Human(friend).into()
            }else{
                Droid(friend).into()
            }
        }).collect::<Vec<_>>();
        let friends_list: Vec<Character> = future::join_all( x).await;
        friends_list
    }

    /// Which movies they appear in.
    async fn appears_in(&self) -> Vec<Episode> {
        self.0.appears_in.clone()
    }

    /// The home planet of the human, or null if unknown.
    async fn home_planet(&self) -> Option<String> {
        self.0.home_planet.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Droid(StarWarsChar);

/// A mechanical creature in the Star Wars universe.
#[Object]
impl Droid {
    /// The id of the droid.
    async fn id(&self) -> String {
        self.0.id.to_string()
    }

    /// The name of the droid.
    async fn name(&self) -> String {
        self.0.name.to_string()
    }

    /// The friends of the droid, or an empty list if they have none.
    async fn friends<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<Character> {
        let redis_client = ctx.data_unchecked::<bb8::Pool<RedisConnectionManager>>();
        let x = self.0.friends.iter().map(|id| async move {
            let mut conn = redis_client.get().await.unwrap();
            let reply: String = get_data_from_redis(&mut *conn, format!("{}*", id)).await.unwrap();
            let friend: StarWarsChar = serde_json::from_str(&reply).unwrap();
            if friend.is_human{
                Human(friend).into()
            }else{
                Droid(friend).into()
            }
        }).collect::<Vec<_>>();
        let friends_list: Vec<Character> = future::join_all( x).await;
        friends_list
    }

    /// Which movies they appear in.
    async fn appears_in(&self) -> Vec<Episode> {
        self.0.appears_in.clone()
    }

    /// The primary function of the droid.
    async fn primary_function(&self) -> Option<String> {
        self.0.primary_function.clone()
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hero<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(
            desc = "If omitted, returns the hero of the whole saga. If provided, returns the hero of that particular episode."
        )]
        episode: Option<Episode>,
    ) -> Character {
        let mut conn = get_redis_conn(&ctx).await;

        match episode {
            Some(episode_name) => {
                if episode_name == Episode::Empire {
                    let reply: String = get_data_from_redis(&mut *conn, "*luke".to_string()).await.unwrap();
                    let luke: StarWarsChar = serde_json::from_str(&reply).unwrap();
                    Human(luke).into()
                } else {
                    let reply: String = get_data_from_redis(&mut *conn, "*artoo".to_string()).await.unwrap();
                    let artoo: StarWarsChar = serde_json::from_str(&reply).unwrap();
                    Droid(artoo).into()
                }
            }
            None => {
                let reply: String = cmd("GET").arg("*luke").query_async(&mut *conn).await.unwrap();
                let luke: StarWarsChar = serde_json::from_str(&reply).unwrap();
                Human(luke).into()
            },
        }
    }

    async fn human<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "id of the human")] id: String,
    ) -> Option<Human> {
        let mut conn = get_redis_conn(&ctx).await;
        let reply: String = get_data_from_redis(&mut *conn, format!("{}*", id)).await.unwrap();
        let data: StarWarsChar = serde_json::from_str(&reply).unwrap();
        Human(data).into()
    }

    async fn humans<'a>(
        &self,
        ctx: &Context<'a>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Human>> {
        let mut conn = get_redis_conn(&ctx).await;
        let keys: Vec<String> = cmd("KEYS")
        .arg("*")
        .query_async(&mut *conn)
        .await?;
        let data: Vec<String> = cmd("MGET")
            .arg(keys)
            .query_async(&mut *conn)
            .await?;
        let data = data.iter().filter(|d| {
             let x: StarWarsChar = serde_json::from_str(d).unwrap();
            x.is_human
        }).map(|d| {
            let x: StarWarsChar = serde_json::from_str(d).unwrap();
            x
        }).collect::<Vec<StarWarsChar>>();
        query_characters(after, before, first, last, data, Human).await
    }

    async fn droid<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "id of the droid")] id: String,
    ) -> Option<Droid> {
        let mut conn = get_redis_conn(&ctx).await;
        let reply: String = get_data_from_redis(&mut *conn, format!("{}*", id)).await.unwrap();
        let data: StarWarsChar = serde_json::from_str(&reply).unwrap();
        Droid(data).into()
    }

    async fn droids<'a>(
        &self,
        ctx: &Context<'a>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Droid>> {
        let mut conn = get_redis_conn(&ctx).await;
        let keys: Vec<String> = cmd("KEYS")
        .arg("*")
        .query_async(&mut *conn)
        .await?;
        let data: Vec<String> = cmd("MGET")
            .arg(keys)
            .query_async(&mut *conn)
            .await?;
        let data = data.iter().filter(|d| {
             let x: StarWarsChar = serde_json::from_str(d).unwrap();
            !x.is_human
        }).map(|d| {
            let x: StarWarsChar = serde_json::from_str(d).unwrap();
            x
        }).collect::<Vec<StarWarsChar>>();
        query_characters(after, before, first, last, data, Droid).await
    }
}

#[derive(Interface, Deserialize, Serialize, Debug)]
#[graphql(
    field(name = "id", type = "String"),
    field(name = "name", type = "String"),
    field(name = "friends", type = "Vec<Character>"),
    field(name = "appears_in", type = "Vec<Episode>")
)]
pub enum Character {
    Human(Human),
    Droid(Droid),
}

async fn query_characters<'a, F, T>(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    characters: Vec<StarWarsChar>,
    map_to: F,
) -> Result<Connection<usize, T>>
where
    F: Fn(StarWarsChar) -> T,
    T: OutputType,
{
    query(
        after,
        before,
        first,
        last,
        |after, before, first, last| async move {
            let mut start = 0usize;
            let mut end = characters.len();

            if let Some(after) = after {
                if after >= characters.len() {
                    return Ok(Connection::new(false, false));
                }
                start = after + 1;
            }

            if let Some(before) = before {
                if before == 0 {
                    return Ok(Connection::new(false, false));
                }
                end = before;
            }

            let mut slice = &characters[start..end];

            if let Some(first) = first {
                slice = &slice[..first.min(slice.len())];
                end -= first.min(slice.len());
            } else if let Some(last) = last {
                slice = &slice[slice.len() - last.min(slice.len())..];
                start = end - last.min(slice.len());
            }

            let mut connection = Connection::new(start > 0, end < characters.len());
            connection.edges.extend(
                slice.to_vec()
                    .iter()
                    .enumerate()
                    .map(move |(idx, item)| Edge::new(start + idx, map_to(item.clone()))),
            );
            Ok::<_, Error>(connection)
        },
    )
    .await
}