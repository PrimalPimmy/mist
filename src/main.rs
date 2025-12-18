use std::collections::{HashMap, VecDeque};
use std::env;

use serenity::all::{ChannelId, MessageId};
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage};
use serenity::model::Timestamp;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    // Map ChannelId to (AuthorName, Content, Timestamp)
    snipes: tokio::sync::Mutex<HashMap<ChannelId, (String, String, Timestamp)>>,
    // Local cache for recent messages
    msg_cache:
        tokio::sync::Mutex<HashMap<ChannelId, VecDeque<(MessageId, String, String, Timestamp)>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Cache the message for snipe functionality
        {
            let mut cache = self.msg_cache.lock().await;
            let channel_msgs = cache.entry(msg.channel_id).or_default();
            channel_msgs.push_back((
                msg.id,
                msg.author.name.clone(),
                msg.content.clone(),
                msg.timestamp,
            ));
            if channel_msgs.len() > 20 {
                channel_msgs.pop_front();
            }
        }

        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "msnipe" {
            let snipes = self.snipes.lock().await;
            if let Some((author, content, timestamp)) = snipes.get(&msg.channel_id) {
                let embed = CreateEmbed::new()
                    .author(CreateEmbedAuthor::new(format!(
                        "Last deleted message by {}",
                        author
                    )))
                    .description(content)
                    .timestamp(*timestamp)
                    .color(0x00ff00); // Green
                let builder = CreateMessage::new().embed(embed);

                if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                    println!("Error sending snipe: {why:?}");
                }
            } else {
                let _ = msg.channel_id.say(&ctx.http, "Nothing to snipe!").await;
            }
        }
    }

    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        msg_id: MessageId,
        _guild_id: Option<serenity::all::GuildId>,
    ) {
        // Attempt to get the message from local cache first, then serenity cache
        let snipe_data = {
            let mut cache = self.msg_cache.lock().await;
            if let Some(msgs) = cache.get_mut(&channel_id) {
                if let Some(idx) = msgs.iter().position(|(id, _, _, _)| *id == msg_id) {
                    let (_, author, content, timestamp) = msgs.remove(idx).unwrap();
                    Some((author, content, timestamp))
                } else {
                    None
                }
            } else {
                None
            }
        };

        let snipe_data = if snipe_data.is_some() {
            snipe_data
        } else if let Some(message) = ctx.cache.message(channel_id, msg_id) {
            Some((
                message.author.name.clone(),
                message.content.clone(),
                message.timestamp,
            ))
        } else {
            None
        };

        if let Some((author, content, timestamp)) = snipe_data {
            let mut snipes = self.snipes.lock().await;
            snipes.insert(channel_id, (author.clone(), content.clone(), timestamp));
            println!("Sniped message in {}: {} - {}", channel_id, author, content);
        } else {
            println!("Message deleted but not found in cache (id={msg_id})");
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            snipes: tokio::sync::Mutex::new(HashMap::new()),
            msg_cache: tokio::sync::Mutex::new(HashMap::new()),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
