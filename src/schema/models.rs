#![allow(clippy::needless_lifetimes)]

use async_graphql::{
    connection::{query, Connection, Edge},
    Context, Enum, Error, Interface, Object, OutputType, Result,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use slab::Slab;

pub type StarWarsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

use bb8_redis::{
    bb8,
    redis::cmd,
    RedisConnectionManager,
};


/// One of the films in the Star Wars Trilogy
#[derive(Enum, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Episode {
    /// Released in 1977.
    NewHope,

    /// Released in 1980.
    Empire,

    /// Released in 1983.
    Jedi,
}

#[derive(Deserialize, Serialize)]
pub struct StarWarsChar {
    id: String,
    name: String,
    is_human: bool,
    friends: Vec<usize>,
    appears_in: Vec<Episode>,
    home_planet: Option<String>,
    primary_function: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Human(StarWarsChar);

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
        // let star_wars = ctx.data_unchecked::<StarWars>();
        // star_wars
        //     .friends(self.0)
        //     .into_iter()
        //     .map(|ch| {
        //         if ch.is_human {
        //             Human(ch).into()
        //         } else {
        //             Droid(ch).into()
        //         }
        //     })
        //     .collect()
        vec![]
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

#[derive(Deserialize, Serialize)]
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
        // let star_wars = ctx.data_unchecked::<StarWars>();
        // star_wars
        //     .friends(self.0)
        //     .into_iter()
        //     .map(|ch| {
        //         if ch.is_human {
        //             Human(ch).into()
        //         } else {
        //             Droid(ch).into()
        //         }
        //     })
        //     .collect()
        vec![]
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
        // let star_wars = ctx.data_unchecked::<StarWars>();
        let redis_client = ctx.data_unchecked::<bb8::Pool<RedisConnectionManager>>();
        let mut conn = redis_client.get().await.unwrap();
        // let reply: String = cmd("GET").arg("foo").query_async(&mut *conn).await.unwrap();
        println!("jdhjffd");

        match episode {
            Some(episode_name) => {
                if episode_name == Episode::Empire {
                    let key: Vec<String> = cmd("KEYS").arg("*luke").query_async(&mut *conn).await.unwrap();
                    let reply: String = cmd("GET").arg(key.get(0)).query_async(&mut *conn).await.unwrap();
                    let luke: StarWarsChar = serde_json::from_str(&reply).unwrap();
                    Human(luke).into()
                } else {
                    // Droid(star_wars.chars.get(star_wars.artoo).unwrap()).into()
                    let reply: String = cmd("GET").arg("*artoo").query_async(&mut *conn).await.unwrap();
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
        // ctx.data_unchecked::<StarWars>().human(&id).map(Human)
        let redis_client = ctx.data_unchecked::<bb8::Pool<RedisConnectionManager>>();
        let mut conn = redis_client.get().await.unwrap();
        let reply: String = cmd("GET").arg("*").query_async(&mut *conn).await.unwrap();
        let data: Vec<StarWarsChar> = serde_json::from_str(&reply).unwrap();
        // data.into()
        None
    }

    async fn humans<'a>(
        &self,
        ctx: &Context<'a>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Human>> {
        // let humans = ctx.data_unchecked::<StarWars>().humans().to_vec();
        // query_characters(after, before, first, last, &humans, Human).await
        Ok(Connection::new(false, false))
    }

    async fn droid<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "id of the droid")] id: String,
    ) -> Option<Droid> {
        // ctx.data_unchecked::<StarWars>().droid(&id).map(Droid)
        None
    }

    async fn droids<'a>(
        &self,
        ctx: &Context<'a>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, Droid>> {
        // let droids = ctx.data_unchecked::<StarWars>().droids().to_vec();
        // query_characters(after, before, first, last, &droids, Droid).await
        Ok(Connection::new(false, false))
    }
}

#[derive(Interface, Deserialize, Serialize)]
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
    characters: &[&'a StarWarsChar],
    map_to: F,
) -> Result<Connection<usize, T>>
where
    F: Fn(&'a StarWarsChar) -> T,
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
                slice
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| Edge::new(start + idx, (map_to)(*item))),
            );
            Ok::<_, Error>(connection)
        },
    )
    .await
}

// ###


// pub struct StarWars {
//     luke: usize,
//     artoo: usize,
//     chars: Slab<StarWarsChar>,
//     chars_by_id: HashMap<String, usize>,
// }
//
// impl StarWars {
//     #[allow(clippy::new_without_default)]
//     pub fn new() -> Self {
//         let mut chars = Slab::new();
//
//         let luke = chars.insert(StarWarsChar {
//             id: "1000".to_string(),
//             name: "Luke Skywalker".to_string(),
//             is_human: true,
//             friends: vec![],
//             appears_in: vec![],
//             home_planet: Some("Tatooine".to_string()),
//             primary_function: None,
//         });
//
//         let vader = chars.insert(StarWarsChar {
//             id: "1001".to_string(),
//             name: "Anakin Skywalker".to_string(),
//             is_human: true,
//             friends: vec![],
//             appears_in: vec![],
//             home_planet: Some("Tatooine".to_string()),
//             primary_function: None,
//         });
//
//         let han = chars.insert(StarWarsChar {
//             id: "1002".to_string(),
//             name: "Han Solo".to_string(),
//             is_human: true,
//             friends: vec![],
//             appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
//             home_planet: None,
//             primary_function: None,
//         });
//
//         let leia = chars.insert(StarWarsChar {
//             id: "1003".to_string(),
//             name: "Leia Organa".to_string(),
//             is_human: true,
//             friends: vec![],
//             appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
//             home_planet: Some("Alderaa".to_string()),
//             primary_function: None,
//         });
//
//         let tarkin = chars.insert(StarWarsChar {
//             id: "1004".to_string(),
//             name: "Wilhuff Tarkin".to_string(),
//             is_human: true,
//             friends: vec![],
//             appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
//             home_planet: None,
//             primary_function: None,
//         });
//
//         let threepio = chars.insert(StarWarsChar {
//             id: "2000".to_string(),
//             name: "C-3PO".to_string(),
//             is_human: false,
//             friends: vec![],
//             appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
//             home_planet: None,
//             primary_function: Some("Protocol".to_string()),
//         });
//
//         let artoo = chars.insert(StarWarsChar {
//             id: "2001".to_string(),
//             name: "R2-D2".to_string(),
//             is_human: false,
//             friends: vec![],
//             appears_in: vec![Episode::Empire, Episode::NewHope, Episode::Jedi],
//             home_planet: None,
//             primary_function: Some("Astromech".to_string()),
//         });
//
//         chars[luke].friends = vec![han, leia, threepio, artoo];
//         chars[vader].friends = vec![tarkin];
//         chars[han].friends = vec![luke, leia, artoo];
//         chars[leia].friends = vec![luke, han, threepio, artoo];
//         chars[tarkin].friends = vec![vader];
//         chars[threepio].friends = vec![luke, han, leia, artoo];
//         chars[artoo].friends = vec![luke, han, leia];
//
//         let chars_by_id = chars.iter().map(|(idx, ch)| (ch.id.to_string(), idx)).collect();
//         Self {
//             luke,
//             artoo,
//             chars,
//             chars_by_id,
//         }
//     }
//
//     pub fn human(&self, id: &str) -> Option<&StarWarsChar> {
//         self.chars_by_id
//             .get(id)
//             .copied()
//             .map(|idx| self.chars.get(idx).unwrap())
//             .filter(|ch| ch.is_human)
//     }
//
//     pub fn droid(&self, id: &str) -> Option<&StarWarsChar> {
//         self.chars_by_id
//             .get(id)
//             .copied()
//             .map(|idx| self.chars.get(idx).unwrap())
//             .filter(|ch| !ch.is_human)
//     }
//
//     pub fn humans(&self) -> Vec<&StarWarsChar> {
//         self.chars
//             .iter()
//             .filter(|(_, ch)| ch.is_human)
//             .map(|(_, ch)| ch)
//             .collect()
//     }
//
//     pub fn droids(&self) -> Vec<&StarWarsChar> {
//         self.chars
//             .iter()
//             .filter(|(_, ch)| !ch.is_human)
//             .map(|(_, ch)| ch)
//             .collect()
//     }
//
//     pub fn friends(&self, ch: &StarWarsChar) -> Vec<&StarWarsChar> {
//         ch.friends
//             .iter()
//             .copied()
//             .filter_map(|id| self.chars.get(id))
//             .collect()
//     }
// }
// ###