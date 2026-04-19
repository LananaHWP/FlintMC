//! Handler for the "msg", "w", and "tell" commands.
use crate::command::arguments::player::PlayerArgument;
use crate::command::arguments::text_component::TextComponentArgument;
use crate::command::commands::{
    CommandExecutor, CommandHandlerBuilder, CommandHandlerDyn, argument,
};
use crate::command::context::CommandContext;
use crate::command::error::CommandError;
use crate::player::Player;
use std::sync::Arc;
use text_components::TextComponent;

#[must_use]
pub fn msg_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(
        &["msg"],
        "Sends a private message to a player.",
        "minecraft:command.msg",
    )
    .then(
        argument("targets", PlayerArgument::multiple())
            .then(argument("message", TextComponentArgument).executes(MsgCommandExecutor)),
    )
}

#[must_use]
pub fn tell_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(
        &["tell"],
        "Sends a private message to a player.",
        "minecraft:command.msg",
    )
    .then(
        argument("targets", PlayerArgument::multiple())
            .then(argument("message", TextComponentArgument).executes(MsgCommandExecutor)),
    )
}

#[must_use]
pub fn w_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(
        &["w"],
        "Sends a private message to a player.",
        "minecraft:command.msg",
    )
    .then(
        argument("targets", PlayerArgument::multiple())
            .then(argument("message", TextComponentArgument).executes(MsgCommandExecutor)),
    )
}

struct MsgCommandExecutor;

impl CommandExecutor<(((), Vec<Arc<Player>>), TextComponent)> for MsgCommandExecutor {
    fn execute(
        &self,
        args: (((), Vec<Arc<Player>>), TextComponent),
        context: &mut CommandContext,
    ) -> Result<(), CommandError> {
        let (((), targets), message) = args;

        for target in &targets {
            let msg = message.clone();
            target.send_message(&msg);
        }

        Ok(())
    }
}