use std::future::Future;
use std::pin::Pin;
use poise::serenity_prelude::{CacheHttp, Context, EventHandler, Reaction, ReactionType, RoleId};
use poise::serenity_prelude::prelude::TypeMapKey;
use crate::events::{get_event_by_manifest, DatabasePool};

pub struct BacchusHandler;

pub struct DBWrapper{
    pub(crate) pool: DatabasePool
}

impl TypeMapKey for DBWrapper{
    type Value= DBWrapper;
}

impl EventHandler for BacchusHandler{
    fn reaction_add<'life0, 'async_trait>(
        &'life0 self,
        ctx: Context,
        add_reaction: Reaction
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait{
        Box::pin(async move {

            //1: Check that the reaction is the right emoji, and in a server
            if add_reaction.emoji != ReactionType::Unicode(String::from("✅")) { return; }
            let guild_id = match add_reaction.guild_id {
                None => { return; }
                Some(id) => id
            };


            //2: Check that there's an event linked to the original message
            let data = ctx.data.read().await;
            let conn = data.get::<DBWrapper>()
                .expect("Shared db could not be found")
                .pool.get().expect("Couldn't connect to Shared DB");

            let (id, event) = match get_event_by_manifest(&conn, u64::from(add_reaction.message_id)){
                Err(_) => { return; }
                Ok((ID, EVENT)) => (ID, EVENT)
            };

            let user_id = add_reaction.user_id.expect("Authorless reaction");
            let user = guild_id.member(ctx.http(), user_id).await.unwrap();

            //3. Add corresponding role to user
            let player_role_id = RoleId::from(event.participant_role_id);
            user.add_role(ctx.http(), player_role_id).await.expect("Error adding role to user");

            println!("Granted {} (id {}) player privileges for event {}(id {}) on {}(id {})",
                user.display_name(),
                user_id,
                event.name,
                id,
                guild_id.name(ctx.cache).unwrap_or(String::from("No server name")),
                guild_id
            );


            //4. TODO: Send message on Discord to inform user of new privileges

        })
    }

    fn reaction_remove<'life0, 'async_trait>(
        &'life0 self,
        ctx: Context,
        remove_reaction: Reaction
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait{
        Box::pin(async move {
            //1: Check that the reaction is the right emoji, and in a server
            if remove_reaction.emoji != ReactionType::Unicode(String::from("✅")) { return; }
            let guild_id = match remove_reaction.guild_id {
                None => { return; }
                Some(id) => id
            };


            //2: Check that there's an event linked to the original message
            let data = ctx.data.read().await;
            let conn = data.get::<DBWrapper>()
                .expect("Shared db could not be found")
                .pool.get().expect("Couldn't connect to Shared DB");

            let (id, event) = match get_event_by_manifest(&conn, u64::from(remove_reaction.message_id)){
                Err(_) => { return; }
                Ok((ID, EVENT)) => (ID, EVENT)
            };

            let user_id = remove_reaction.user_id.expect("Authorless reaction");
            let user = guild_id.member(ctx.http(), user_id).await.unwrap();

            //3. Add corresponding role to user
            let player_role_id = RoleId::from(event.participant_role_id);
            user.remove_role(ctx.http(), player_role_id).await.expect("Error adding role to user");

            println!("Stripped {} (id {}) of player privileges for event {}(id {}) on {}(id {})",
                     user.display_name(),
                     user_id,
                     event.name,
                     id,
                     guild_id.name(ctx.cache).unwrap_or(String::from("No server name")),
                     guild_id
            );
        })
    }
}