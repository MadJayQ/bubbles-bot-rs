#[macro_use]
extern crate lazy_static;

mod database;

use std::{collections::HashSet, env, sync::Arc};
// use commands::{request::*};

// use commands::{math::*, meta::*, owner::*};
use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::{
            macros::{command, group, hook},
            Args, CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready},
};

use tracing::{error, info};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[command]
#[bucket = "request"]
async fn gold(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount = args.single::<i64>()?;
    // msg.channel_id.say(&ctx.http, format!("{:?} is requesting {} gold!", msg.author, amount)).await?;
    let msg_result = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Bank Request").description(format!(
                    "{} is requesting {} gold!",
                    &msg.author.name, amount
                )).thumbnail("https://wow.zamimg.com/images/wow/icons/large/inv_misc_coin_02.jpg")
                .footer(|f| f.text("Please wait for officer approval"))
            })
        })
        .await;

    if let Err(why) = msg_result {
        println!("Error sending msg: {:?}", why);
    }

    let user = database::RequestUser{
        name: String::from(&msg.author.name),
        user_id: msg.author.id.0 as i64
    };

    if let Some(db) = database::DATABASE.get() {
        db.add_gold_request(amount, user).await;
    }

    Ok(())
}

#[command]
#[bucket = "request"]
async fn materials(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, format!("Requesting materials! {:?}", args))
        .await?;
    let db = &database::DATABASE;
    // let entry = sqlx::query!

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "This is a small test-bot! : )")
        .await?;

    Ok(())
}

#[group]
#[prefix = "request"]
#[description = ""]
#[summary = ""]
#[default_command(gold)]
#[commands(gold, materials)]
struct Request;

#[group]
#[commands(about)]
struct General;

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let database = std::thread::spawn(|| {
        database::connect()
    }).join().expect("Thread panicked!");
    database::DATABASE.set(database.await).unwrap();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .normal_message(normal_message)
        .unrecognised_command(unknown_command)
        .bucket("request", |b| b.delay(5))
        .await
        .group(&REQUEST_GROUP)
        .group(&GENERAL_GROUP);

    // let db =

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .application_id(u64::from(bot_id))
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
