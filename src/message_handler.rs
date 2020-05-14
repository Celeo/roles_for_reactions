use crate::shared::{ReactionRole, StateManager};
use log::{debug, error, info};
use serenity::{client::Context, model::channel::Message};
use std::error::Error;

pub(crate) fn message_handler(ctx: Context, message: Message) -> Result<(), Box<dyn Error>> {
    if message.guild_id.is_some() {
        // in guilds, users use the commands to interact with the bot
        return Ok(());
    }
    // return if author is a bot
    if message.author.bot {
        debug!("Abort message_handler: author is bot");
        return Ok(());
    }

    // get manager
    let mut data = ctx.data.write();
    let state_manager = data
        .get_mut::<StateManager>()
        .expect("Could not get state_manager from context");

    // allow user to quit setup
    if message.content.to_lowercase() == "quit" {
        state_manager.remove(&message.author.name);
        message.reply(&ctx, "Setup terminated.")?;
        return Ok(());
    }

    // get current state; return if there isn't any
    let state = match state_manager.get_mut(&message.author.name) {
        Some(s) => s,
        None => return Ok(()),
    };

    // if no post content, the first message is that
    if state.post_content.is_none() {
        debug!("Adding post_content to state");
        state.post_content = Some(message.content.clone());
        message.reply(
            &ctx,
            "Got it.\n\nNow, enter an emoji and the role name, 1 pair per \
message with a space between, like [emoji] [role name]. Send a 'done' message when done.",
        )?;
        return Ok(());
    }

    if message.content.to_lowercase() == "done" {
        info!(
            "Adding new monitor for guild_id {} and channel_id {}",
            state.guild_id, state.channel_id
        );
        state_manager.remove(&message.author.name);
        message.reply(&ctx, "All done! See the post in the channel.")?;
        // TODO make post in the configured channel including the emojis, etc.
        // TODO save the state somewhere
        return Ok(());
    }

    // get emoji and role name from message
    let mut chars = message.content.chars();
    let emoji = match chars.next() {
        Some(e) => e,
        None => {
            message.reply(
                &ctx,
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
            message.reply(&ctx, "Could not find your guild!")?;
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
            &ctx,
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
    message.reply(&ctx, "Got it. Enter another, or 'done' to finish")?;

    Ok(())
}
