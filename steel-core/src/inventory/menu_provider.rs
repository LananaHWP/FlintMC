//! Menu provider for opening menus.

use steel_registry::menu_type::MenuTypeRef;
use text_components::TextComponent;

use crate::inventory::menu::Menu;
use crate::player::Player;
use crate::inventory::lock::ContainerLockGuard;

/// Trait for menu instances that can be opened by players.
///
/// This extends `Menu` with the additional information needed to send
/// the open screen packet: menu type and container ID.
pub trait MenuInstance: Menu + Send + Sync {
    /// Returns the menu type for the open screen packet.
    fn menu_type(&self) -> MenuTypeRef;

    /// Returns the container ID for this menu.
    fn container_id(&self) -> u8;

    /// Handles a button click in this menu.
    /// Returns true if the click was handled and changes were made.
    fn on_button_click(&mut self, _player: &Player, _guard: &mut ContainerLockGuard, _button_id: i32) -> bool {
        false
    }
}

/// Trait for types that can create menus.
///
/// Each menu type implements this with a struct that holds the necessary data.
pub trait MenuProvider {
    /// Returns the display title for this menu.
    fn title(&self) -> TextComponent;

    /// Creates a menu with the given container ID.
    fn create(&self, container_id: u8) -> Box<dyn MenuInstance>;
}
