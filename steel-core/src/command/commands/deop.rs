//! Handler for the "deop" command.
use std::sync::Arc;

use crate::{
    command::{
        arguments::player::PlayerArgument,
        commands::{CommandHandlerBuilder, CommandHandlerDyn, argument},
        context::CommandContext,
    },
    player::Player,
};
use steel_protocol::packets::game::CSystemChat;
use text_components::TextComponent;

/// Handler for the "deop" command.
#[must_use]
pub fn command_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(&["deop"], "Removes operator status from a player.", "minecraft:command.deop")
        .then(
            argument("targets", PlayerArgument::multiple())
                .executes(
                    |((), targets): ((), Vec<Arc<Player>>), ctx: &mut CommandContext| {
                        // Get mutable access to permissions
                        let mut permissions = ctx.server.permissions.write();
                        
                        // Remove operator status from each target
                        for target in &targets {
                            let current_level = target.permission_level();
                            target.set_permission_level(0);
                            
                            // Update in the permission manager
                            permissions.remove_operator(target.gameprofile.id);
                            
                            log::info!("Removed operator status from {} (was level {})", target.gameprofile.name, current_level);
                            
                            // Notify the target player
                            target.send_packet(CSystemChat {
                                content: TextComponent::from("You are no longer an operator"),
                                overlay: false,
                            });
                        }
                        
                        // Save the operators file
                        if let Err(e) = permissions.save() {
                            log::error!("Failed to save ops.json: {}", e);
                        }
                        
                        // Send message to sender
                        if let Some(sender_player) = ctx.sender.get_player() {
                            sender_player.send_packet(CSystemChat {
                                content: TextComponent::from(format!(
                                    "Removed operator status from {}",
                                    targets.iter()
                                        .map(|p| p.gameprofile.name.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )),
                                overlay: false,
                            });
                        }
                        
                        Ok(())
                    },
                )
        )
}