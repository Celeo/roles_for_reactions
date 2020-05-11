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
use std::{collections::HashSet, env};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _context: Context, _ready: Ready) {
        info!("Bot connected");
    }

    fn reaction_add(&self, _ctx: Context, add_reaction: Reaction) {
        // TODO
        debug!("Reaction added: {:?}", add_reaction);
    }
}

#[group]
#[commands(setup)]
struct General;

#[command]
#[description = "Setup a new post to watch"]
fn setup(context: &mut Context, message: &Message) -> CommandResult {
    // TODO
    message.reply(
        &context,
        "I see you, but this command isn't yet implemented.",
    )?;
    Ok(())
}

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
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "roles_for_reactions");
    }
    pretty_env_logger::init();
    debug!("Loading env");
    kankyo::init().expect("Could not load .env file");
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN environment variable");

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

    debug!("Starting bot");
    if let Err(err) = client.start() {
        error!("Error starting bot: {:?}", err);
    }
}
