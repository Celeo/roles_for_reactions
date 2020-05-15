use log::{debug, error, info};
use serenity::{
    client::Context,
    framework::{
        standard::{
            help_commands,
            macros::{command, group, help},
            Args, CommandGroup, CommandResult, HelpOptions,
        },
        StandardFramework,
    },
    model::{
        channel::{Message, Reaction},
        gateway::Ready,
        id::UserId,
    },
    prelude::EventHandler,
    Client,
};
use std::{
    collections::{HashMap, HashSet},
    env, process,
};

mod message_handler;
use message_handler::message_handler;
mod reaction_handler;
use reaction_handler::reaction_handler;
mod shared;
use shared::{MonitorManager, SetupState, StateManager};

/// Struct for implementing the `EventHandler` trait on for
/// receiving events.
struct Handler;

impl EventHandler for Handler {
    /// Called when the bot is ready to receive events.
    fn ready(&self, _context: Context, _ready: Ready) {
        info!("Bot connected");
    }

    /// Called when a reaction is added to a message the bot can see.
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if let Err(e) = reaction_handler(ctx, add_reaction) {
            error!("Could not process message: {}", e);
        };
    }

    /// Called when a message is added that the bot can see.
    fn message(&self, ctx: Context, message: Message) {
        if let Err(e) = message_handler(ctx, message) {
            error!("Could not process message: {}", e);
        };
    }
}

/// Command group.
#[group]
#[commands(setup)]
struct General;

/// Command handler for 'setup'.
///
/// Sets the state for the user to the starting state, recording
/// the channel id and guild id that the command was used in.
#[command]
#[description = "Setup a new post to watch"]
fn setup(ctx: &mut Context, message: &Message) -> CommandResult {
    if message.guild_id.is_none() {
        // command only works in channels, not DMs
        return Ok(());
    }

    // TODO check that the user making the request has the 'manage roles' permissions.

    message.reply(&ctx, "Let's do it! Check your DMs.")?;
    let mut data = ctx.data.write();
    let manager = data
        .get_mut::<StateManager>()
        .expect("Could not get StateManager from context");
    manager.insert(
        message.author.name.clone(),
        SetupState::new(
            *message.channel_id.as_u64(),
            *message.guild_id.unwrap().as_u64(),
        ),
    );
    message.author.direct_message(&ctx, |m| {
        m.content(format!(
            "Setup post in '{}'. Enter the content of the post as a reply to this.",
            message.channel(&ctx).unwrap()
        ))
    })?;
    Ok(())
}

// Configure the bot to use the built-in help.
#[help]
fn bot_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

/// Entry point.
fn main() {
    kankyo::init().expect("Could not load .env file");
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "roles_for_reactions");
    }
    pretty_env_logger::init();
    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Missing DISCORD_TOKEN environment variable");
            process::exit(1);
        }
    };

    debug!("Creating client");
    let mut client = Client::new(&token, Handler).expect("Could not create client");

    debug!("Getting bot info");
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(err) => panic!("Could not access application info: {:?}", err),
    };

    debug!("Configuring commands");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("!rfr ")
                    .on_mention(Some(bot_id))
                    .case_insensitivity(true)
                    .owners(owners)
            })
            .after(|_context, _message, command_name, result| {
                if let Err(error) = result {
                    error!("Error in command '{}': {:?}", command_name, error);
                }
            })
            .unrecognised_command(|_, _, name| {
                debug!("Got unrecognized command '{}'", name);
            })
            .on_dispatch_error(|_context, message, error| {
                error!(
                    "Command error occurred in '{}': {:?}",
                    message.content, error
                );
            })
            .group(&GENERAL_GROUP)
            .help(&BOT_HELP),
    );

    debug!("Setting up memory");
    {
        let mut data = client.data.write();
        data.insert::<StateManager>(HashMap::new());
        let monitor_data = match MonitorManager::load() {
            Ok(d) => d,
            Err(e) => {
                error!("Could not load monitor configuration: {}", e);
                process::exit(1);
            }
        };
        data.insert::<MonitorManager>(monitor_data);
    }

    debug!("Starting bot");
    if let Err(err) = client.start() {
        error!("Error starting bot: {:?}", err);
    }
}
