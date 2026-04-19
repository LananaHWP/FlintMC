//! Handler for the "team" command.
use crate::command::commands::{
    CommandExecutor, CommandHandlerBuilder, CommandHandlerDyn, literal,
};
use crate::command::context::CommandContext;
use crate::command::error::CommandError;

#[must_use]
pub fn command_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(
        &["team"],
        "Manages teams.",
        "minecraft:command.team",
    )
    .then(literal("list").then(
        literal("team").executes(TeamListOneExecutor),
    ))
    .then(literal("list").executes(TeamListExecutor))
}

struct TeamListExecutor;

impl CommandExecutor<()> for TeamListExecutor {
    fn execute(&self, _args: (), context: &mut CommandContext) -> Result<(), CommandError> {
        Ok(())
    }
}

struct TeamListOneExecutor;

impl CommandExecutor<()> for TeamListOneExecutor {
    fn execute(&self, _args: (), context: &mut CommandContext) -> Result<(), CommandError> {
        Ok(())
    }
}