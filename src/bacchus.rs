#![allow(dead_code)]

use std::iter;
use std::str::FromStr;
use crate::events::{delete_event, get_all_events, get_channels_by_event_id, get_event_by_channel, insert_channels, insert_event, DatabasePool, EventData};
use futures::future::try_join_all;
use poise::CreateReply;
use poise::serenity_prelude::{Attachment, Channel, ChannelType, CreateChannel, CreateEmbed, CreateEmbedFooter, CreateMessage, EditRole, MessageId, PermissionOverwrite, PermissionOverwriteType, Permissions, Role, RoleId};
use poise::serenity_prelude::ChannelId;

pub struct Data {
    pub(crate) conn: DatabasePool,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command, subcommands("create", "delete", "list"))]
pub async fn event(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("I am a prefix command").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "How shall this event be named ?"] name: String,

    #[description = "(Optional) Provide a short description of the event"]
    short_description: Option<String>,

    #[description = "Describe the proceedings"] description: Option<String>,

    #[description = "A thumbnail for your event."] thumbnail: Option<Attachment>,

    #[description = "A picture for your event."] picture: Option<Attachment>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or(Error::from("That command can only be ran in a server"))?;
    let http = ctx.http();

    let created_roles: Vec<Role> = try_join_all(vec![
        guild_id.create_role(
            ctx.http(),
            EditRole::new().name(format!("{}-manager", name)),
        ),
        guild_id.create_role(ctx.http(), EditRole::new().name(format!("{}-player", name))),
    ])
    .await
    .map_err(Error::from)?
    .into_iter()
    .collect();

    let (manager, player) = (&created_roles[0], &created_roles[1]);

    let member = guild_id.member(&http, ctx.author().id).await?;
    member.add_role(&http, manager).await?;

    let everyone_role = guild_id
        .roles(http)
        .await?
        .values()
        .find(|r| r.name == "@everyone")
        .ok_or_else(|| Error::from("Could not find @everyone role"))?
        .clone();

    let channel_permissions = vec![
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(player.id),
        },
        PermissionOverwrite {
            allow: Permissions::MANAGE_CHANNELS
                | Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(manager.id),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL | Permissions::MANAGE_CHANNELS | Permissions::SEND_MESSAGES,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(ctx.cache().current_user().id)
        }
    ];

    let category = guild_id.create_channel(
        http,
        CreateChannel::new(&name).permissions(channel_permissions.clone()).kind(ChannelType::Category)
    ).await?;

    // Create channel
    let general_channel = guild_id
        .create_channel(
            http,
            CreateChannel::new("general").permissions(channel_permissions).category(category.id),
        )
        .await?;

    let mut embed = CreateEmbed::new()
        .title(&name)
        .description(description.clone().unwrap_or_default())
        .field("Creator", &ctx.author().name, true)
        .footer(CreateEmbedFooter::new("React with ✅ to join the event"));

    if let Some(thumb) = &thumbnail {
        embed = embed.thumbnail(thumb.clone().url)
    }

    let builder = CreateMessage::new().embed(embed);
    let answer = ctx.channel_id().send_message(ctx.http(), builder).await?;

    answer.react(ctx.http(), '✅').await?;

    let event_id = insert_event(
        &ctx.data().conn.get().unwrap(),
        EventData {
            name,
            short_description,
            description,
            thumbnail: thumbnail.map(|x| x.url),
            picture: picture.map(|x| x.url),
            max_participants: None,

            server_id: u64::from(guild_id),
            manager_role_id: u64::from(manager.id),
            participant_role_id: u64::from(player.id),
            manifest_id: u64::from(answer.id),
            category_id: u64::from(category.id)
        },
    )?;

    insert_channels(&ctx.data().conn.get().unwrap(), event_id, vec![u64::from(general_channel.id)])?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn delete(ctx: Context<'_>) -> Result<(), Error> {

    let (id, event) = get_event_by_channel(&ctx.data().conn.get().unwrap(), u64::from(ctx.channel_id()))
        .expect("Failed to get related event (are you running this in a managed event channel ?)");
    let http = ctx.http();
    let guild_id = ctx.guild_id().expect("This command can only be ran in a server");

    ctx.reply(format!("Seleting event {}. Goodbye !", event.name)).await?;

    //TODO: Delete

    try_join_all(vec![
        guild_id.delete_role(http, RoleId::from(event.manager_role_id)),
        guild_id.delete_role(http, RoleId::from(event.participant_role_id))
    ]).await?;

    //Delete owned channels + category
    let channels_ids = get_channels_by_event_id(&ctx.data().conn.get().expect("Couldnt get a reference to database"), id)?;
    try_join_all(channels_ids.iter().chain(vec![event.category_id].iter()).map(|x| ChannelId::new(*x).delete(http) )).await?;

    delete_event(&ctx.data().conn.get().unwrap(), id)?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {

    let db = ctx.data().conn.get()?;
    let event_store = get_all_events(&db)?;

    if event_store.len() == 0 {
        ctx.reply("No message").await?;
    } else {
       let body = event_store.iter().map(|(id, event)| format!("{}", event.name).to_string()).collect::<Vec<String>>().join("\n");
        ctx.reply(body).await?;
    }

    Ok(())
}
