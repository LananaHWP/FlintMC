//! Handler for the "op" command.
use std::sync::Arc;

use crate::{
    command::{
        arguments::player::PlayerArgument,
        commands::{CommandHandlerBuilder, CommandHandlerDyn, argument},
        context::CommandContext,
    },
    permissions::PermissionLevel,
    player::Player,
};
use steel_protocol::packets::game::CSystemChat;
use text_components::TextComponent;

/// Handler for the "op" command.
#[must_use]
pub fn command_handler() -> impl CommandHandlerDyn {
    CommandHandlerBuilder::new(&["op"], "Makes a player an operator.", "minecraft:command.op")
        .then(
            argument("targets", PlayerArgument::multiple())
                .executes(
                    |((), targets): ((), Vec<Arc<Player>>), ctx: &mut CommandContext| {
                        // Get mutable access to permissions
                        let mut permissions = ctx.server.permissions.write();
                        
                        // Set each target to operator level 4
                        for target in &targets {
                            let current_level = target.permission_level();
                            target.set_permission_level(PermissionLevel::Level4.as_i32());
                            
                            // Update in the permission manager
                            permissions.add_operator(
                                target.gameprofile.id,
                                target.gameprofile.name.clone(),
                                PermissionLevel::Level4,
                            );
                            
                            log::info!("Made {} an operator (was level {})", target.gameprofile.name, current_level);
                            
                            // Notify the target player
                            target.send_packet(CSystemChat {
                                content: TextComponent::from("You are now an operator"),
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
                                    "Made {} an operator",
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