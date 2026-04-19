//! Handler for the "scoreboard" command.
use crate::command::commands::{
    CommandExecutor, CommandHandlerBuilder, CommandHandlerDyn, literal,
};
use crate::command::context::CommandContext;
use crate::command::error::CommandError;

#[must_use]
pub fn command_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(
        &["scoreboard"],
        "Manages scoreboard objectives and players.",
        "minecraft:command.scoreboard",
    )
    .then(literal("objectives").then(
        literal("list").executes(ScoreboardObjectivesListExecutor),
    ))
    .then(literal("players").then(
        literal("list").executes(ScoreboardPlayersListExecutor),
    ))
}

struct ScoreboardObjectivesListExecutor;

impl CommandExecutor<()> for ScoreboardObjectivesListExecutor {
    fn execute(&self, _args: (), context: &mut CommandContext) -> Result<(), CommandError> {
        Ok(())
    }
}

struct ScoreboardPlayersListExecutor;

impl CommandExecutor<()> for ScoreboardPlayersListExecutor {
    fn execute(&self, _args: (), context: &mut CommandContext) -> Result<(), CommandError> {
        Ok(())
    }
}