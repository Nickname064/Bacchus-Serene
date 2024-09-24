mod bacchus;
mod events;
mod bacchus_handler;

use crate::bacchus::{event, init, Data};
use crate::events::{create_tables, DatabasePool};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GatewayIntents;
use crate::bacchus_handler::{BacchusHandler, DBWrapper};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::MESSAGE_CONTENT;

    let conn = DatabasePool::new(&std::env::args().nth(1).expect("Specify a database path"))
        .expect("Failed to open db");
    create_tables(&conn.get().unwrap()).expect("Couldn't initialize tables");

    let conn2 = conn.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![event(), init()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { conn: conn2 }) //Share the db with the command handlers
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(BacchusHandler)
        .await
        .expect("Error creating client");

    // Share the DB with the event handlers
    let mut data = client.data.write().await;
    data.insert::<DBWrapper>(DBWrapper{pool: conn});
    drop(data);

    client.start().await.unwrap();
}
