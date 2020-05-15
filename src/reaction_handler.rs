use crate::shared::{MonitorManager, ReactionRole};
use log::error;
use serenity::{
    client::Context,
    model::{
        channel::{Reaction, ReactionType},
        guild::{Member, Role},
    },
};
use std::error::Error;

/// Turn a guild_id and user_id into a `Member`.
fn get_member_by_user_id(
    ctx: &Context,
    guild_id: u64,
    user_id: u64,
) -> Result<Member, Box<dyn Error>> {
    let members = ctx.http.get_guild_members(guild_id, None, None)?;
    let matching_members = members
        .iter()
        .filter(|&m| m.user_id().as_u64() == &user_id)
        .collect::<Vec<&Member>>();
    let member = matching_members.first().ok_or(format!(
        "Could not find member with matching user id {}",
        user_id
    ))?;
    Ok(member.to_owned().to_owned())
}

/// Turn a guid_id and role_name into a `Role`.
fn get_role_by_name(ctx: &Context, guild_id: u64, role_name: &str) -> Result<Role, Box<dyn Error>> {
    let roles = ctx.http.get_guild_roles(guild_id)?;
    let matching_roles = roles
        .iter()
        .filter(|r| r.name == role_name)
        .collect::<Vec<&Role>>();
    let role = matching_roles.first().ok_or(format!(
        "Could not find role with matching name {}",
        role_name
    ))?;
    Ok(role.to_owned().to_owned())
}

/// Add a role to a `Member` from a `ReactionRole`.
fn add_role_by_reaction(
    ctx: &Context,
    reaction: &ReactionRole,
    member: &mut Member,
) -> Result<(), Box<dyn Error>> {
    let role_ref = get_role_by_name(ctx, *member.guild_id.as_u64(), &reaction.role_name)?;
    member.add_role(ctx, role_ref)?;
    Ok(())
}

/// Handle any reactions that the bot sees.
pub(crate) fn reaction_handler(ctx: Context, reaction: Reaction) -> Result<(), Box<dyn Error>> {
    let guild_id = match reaction.guild_id {
        Some(g) => g,
        None => {
            // not watching reactions outside of guild channels
            return Ok(());
        }
    };
    let reaction_str = match reaction.emoji {
        ReactionType::Unicode(ref s) => s.to_owned(),
        _ => {
            error!("Incompatible type");
            return Ok(());
        }
    };
    let mut member = get_member_by_user_id(&ctx, *guild_id.as_u64(), *reaction.user_id.as_u64())?;

    let mut data = ctx.data.write();
    let monitor_manager = data
        .get_mut::<MonitorManager>()
        .expect("Could not get monitor_manager from context");
    for monitor in monitor_manager {
        if monitor.channel_id == *reaction.channel_id.as_u64() {
            for saved_reaction in &monitor.reactions {
                let sr_string = format!("{}", saved_reaction.emoji);
                if sr_string == reaction_str {
                    add_role_by_reaction(&ctx, &saved_reaction, &mut member)?;
                }
            }
        }
    }

    Ok(())
}
