use crate::shared::{Monitor, MonitorManager, ReactionRole, StateManager};
use log::{debug, error, info};
use serenity::{
    client::Context,
    model::channel::Message,
    model::channel::{Channel, ReactionType},
    utils::MessageBuilder,
};
use std::error::Error;

/// Return whether the message should be ignored.
fn should_ignore(message: &Message) -> bool {
    if message.guild_id.is_some() {
        // in guilds, users use the commands to interact with the bot
        return true;
    }
    // return if author is a bot
    if message.author.bot {
        return true;
    }
    false
}

/// Handle the user sending the 'quit' message to abort the setup.
fn handle_quit(ctx: &Context, message: &Message) -> Result<(), Box<dyn Error>> {
    let mut data = ctx.data.write();
    let state_manager = data
        .get_mut::<StateManager>()
        .expect("Could not get state_manager from context");
    state_manager.remove(&message.author.name);
    message.reply(ctx, "Setup terminated.")?;
    Ok(())
}

/// Handle the user sending the 'done' message to finish the setup.
fn handle_done(ctx: &Context, message: &Message) -> Result<(), Box<dyn Error>> {
    debug!("Process 'done' message");
    let mut data = ctx.data.write();
    let state_manager = data
        .get_mut::<StateManager>()
        .expect("Could not get state_manager from context");

    debug!("Removing setup from manager");
    let state = match state_manager.remove(&message.author.name) {
        Some(s) => s,
        None => {
            message.reply(
                ctx,
                "Sorry, something went wrong and the request wasn't able to be finished.",
            )?;
            error!("Could not find state in state manager to finish");
            return Ok(());
        }
    };

    // unwrap here is safe because of the logic in 'message_handler'
    let to_post = MessageBuilder::new()
        .push(state.post_content.unwrap())
        .build();

    debug!("Posting new message");
    let channel: Channel = ctx.http.get_channel(state.channel_id)?;
    let posted_message = match channel {
        Channel::Group(_) => match channel.group() {
            Some(lock) => lock.read().say(ctx, to_post)?,
            None => {
                error!("Could not retrieve the channel by id {}", state.channel_id);
                message.reply(ctx, "The channel reference could not be retrieved")?;
                return Ok(());
            }
        },
        Channel::Private(_) => match channel.private() {
            Some(lock) => lock.read().say(ctx, to_post)?,
            None => {
                error!("Could not retrieve the channel by id {}", state.channel_id);
                message.reply(ctx, "The channel reference could not be retrieved")?;
                return Ok(());
            }
        },
        Channel::Guild(_) => match channel.guild() {
            Some(lock) => lock.read().say(ctx, to_post)?,
            None => {
                error!("Could not retrieve the channel by id {}", state.channel_id);
                message.reply(ctx, "The channel reference could not be retrieved")?;
                return Ok(());
            }
        },
        _ => {
            error!("Unsupported channel type {}", channel);
            message.reply(ctx, "The channel type was not recognized")?;
            return Ok(());
        }
    };

    // add the reactions to the message
    for reaction in &state.reactions {
        ctx.http.create_reaction(
            state.channel_id,
            *posted_message.id.as_u64(),
            &ReactionType::Unicode(format!("{}", reaction.emoji)),
        )?;
    }

    debug!("Adding to monitor manager and saving to config file");
    let monitor_manager = data
        .get_mut::<MonitorManager>()
        .expect("Could not get monitor_manager from context");
    let new_monitor = Monitor::new(
        state.channel_id,
        state.guild_id,
        *posted_message.id.as_u64(),
        &state.reactions,
    );
    monitor_manager.push(new_monitor);
    MonitorManager::save(monitor_manager)?;

    info!("Successfully processed 'done' message");
    message.reply(ctx, "All done! See the post in the channel.")?;
    Ok(())
}

/// Handle the interview process of getting messages from the user
/// and adding the `post_content` field and reactions to the `SetupState`
/// struct.
fn handle_interview(ctx: &Context, message: &Message) -> Result<(), Box<dyn Error>> {
    // get current state; ignore message if there isn't any
    let mut data = ctx.data.write();
    let state_manager = data
        .get_mut::<StateManager>()
        .expect("Could not get state_manager from context");
    let state = match state_manager.get_mut(&message.author.name) {
        Some(s) => s,
        None => return Ok(()),
    };

    // if no post content, the first message is that
    if state.post_content.is_none() {
        debug!("Adding post_content to state");
        state.post_content = Some(message.content.clone());
        message.reply(
            ctx,
            "Got it.\n\nNow, enter an emoji and the role name, 1 pair per \
message with a space between, like [emoji] [role name]. Send a 'done' message when done.",
        )?;
        return Ok(());
    }

    // get emoji and role name from message
    let mut chars = message.content.chars();
    let emoji = match chars.next() {
        Some(e) => e,
        None => {
            message.reply(
                ctx,
                "Doesn't look like the message format was right - it's [emoji] [role name]",
            )?;
            return Ok(());
        }
    };
    let role_name: String = chars.skip(1).collect();

    // get guild for validation
    let guild = match ctx.http.get_guild(state.guild_id) {
        Ok(g) => g,
        Err(e) => {
            error!("Could not find guild by id: {}", e);
            message.reply(ctx, "Could not find your guild!")?;
            return Ok(());
        }
    };

    // match role_name against actual roles
    let all_role_names = guild
        .roles
        .values()
        .map(|r| r.name.clone())
        .filter(|name| name != "@everyone")
        .collect::<Vec<String>>();
    let matching_roles = all_role_names
        .iter()
        .filter(|&r| r == &role_name)
        .take(1)
        .collect::<Vec<_>>();
    if matching_roles.first().is_none() {
        debug!(
            "User supplied role_name {} but that didn't match a valid role",
            role_name
        );
        message.reply(
            ctx,
            format!(
                "Could not find that role. Valid role names are {}",
                all_role_names.join(", ")
            ),
        )?;
        return Ok(());
    }

    // store the reaction + role
    debug!("Pushing new ReactionRole");
    state.reactions.push(ReactionRole::new(emoji, &role_name));
    message.reply(ctx, "Got it. Enter another, or 'done' to finish")?;

    Ok(())
}

/// Handle any messages that the bot sees.
pub(crate) fn message_handler(ctx: Context, message: Message) -> Result<(), Box<dyn Error>> {
    if should_ignore(&message) {
        return Ok(());
    }

    if message.content.to_lowercase() == "quit" {
        return handle_quit(&ctx, &message);
    }

    if message.content.to_lowercase() == "done" {
        return handle_done(&ctx, &message);
    }

    handle_interview(&ctx, &message)
}
