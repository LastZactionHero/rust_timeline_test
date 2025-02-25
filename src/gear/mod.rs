mod score_editor;

use crate::draw_components::{DrawComponent, DrawResult, Position};
use crate::events::InputEvent;

pub use score_editor::ScoreEditorGear;

/// Trait that defines the interface for all gear types
pub trait Gear {
    /// Returns a reference to the draw component for this gear
    fn get_draw_component(&self) -> Box<dyn DrawComponent>;
    
    /// Handle an input event, return true if event was handled
    fn handle_event(&mut self, event: &InputEvent) -> bool;
    
    /// Get the name of this gear for display purposes
    fn name(&self) -> &'static str;
}

/// Enum to represent the different types of gear available in the application
#[derive(Clone)]
pub enum GearType {
    ScoreEditor,
    Mixer,
    // Add more gear types as needed
}