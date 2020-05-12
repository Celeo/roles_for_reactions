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
    prelude::{EventHandler, TypeMapKey},
    Client,
};
use std::{
    collections::{HashMap, HashSet},
    env, process,
};

#[derive(Debug)]
struct ReactionRole {
    emoji_id: String,
    role_name: String,
}

#[derive(Debug)]
struct SetupState {
    channel_id: u64,
    guild_id: u64,
    post_content: Option<String>,
    reactions: Vec<ReactionRole>,
}

impl SetupState {
    fn new(channel_id: u64, guild_id: u64) -> Self {
        Self {
            channel_id: channel_id.to_owned(),
            guild_id: guild_id.to_owned(),
            post_content: None,
            reactions: Vec::new(),
        }
    }
}

struct StateManager;

impl TypeMapKey for StateManager {
    type Value = HashMap<String, SetupState>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _context: Context, _ready: Ready) {
        info!("Bot connected");
    }

    fn reaction_add(&self, _ctx: Context, add_reaction: Reaction) {
        debug!("Reaction added: {:?}", add_reaction);
        // ...
    }

    fn message(&self, ctx: Context, message: Message) {
        if message.guild_id.is_some() {
            // in guilds, users use the commands to interact with the bot
            return;
        }
        // return if author is a bot
        if message.author.bot {
            return;
        }
        // get map
        let mut data = ctx.data.write();
        let manager = data
            .get_mut::<StateManager>()
            .expect("Could not get manager from context");

        // allow user to quit setup
        if message.content.to_lowercase() == "quit" {
            manager.remove(&message.author.name);
            if let Err(e) = message.reply(&ctx, "Setup terminated.") {
                error!("Could not tell user setup was terminated: {}", e);
            };
            return;
        }

        let state = match manager.get_mut(&message.author.name) {
            Some(s) => s,
            None => return,
        };

        // if no post content, the first message is that
        if state.post_content.is_none() {
            state.post_content = Some(String::from(""))
        }
    }
}

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
    }

    debug!("Starting bot");
    if let Err(err) = client.start() {
        error!("Error starting bot: {:?}", err);
    }
}
