// use crate::shared::{Monitor, MonitorManager};
use log::debug;
use serenity::{client::Context, model::channel::Reaction};
use std::error::Error;

// TODO

/// Handle any reactions that the bot sees.
pub(crate) fn reaction_handler(_ctx: Context, reaction: Reaction) -> Result<(), Box<dyn Error>> {
    debug!("Reaction added: {:?}", reaction);
    Ok(())
}
