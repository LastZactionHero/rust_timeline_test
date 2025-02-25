mod track_editor;  // Module for track editing
mod score_editor;  // Module for score editing (all instruments)

use crate::draw_components::{DrawComponent, DrawResult, Position, ViewportDrawResult};
use crate::events::InputEvent;

pub use track_editor::TrackEditorGear;  // Export the track editor gear
pub use score_editor::ScoreEditorGear;  // Export the score editor gear

/// Trait that defines the interface for all gear types
pub trait Gear {
    /// Returns a reference to the draw component for this gear
    fn get_draw_component(&self) -> Box<dyn DrawComponent>;
    
    /// Handle an input event, return true if event was handled
    fn handle_event(&mut self, event: &InputEvent) -> bool;
    
    /// Get the name of this gear for display purposes
    fn name(&self) -> &'static str;
    
    /// Set the viewport draw result from the last render
    fn set_viewport_draw_result(&mut self, result: ViewportDrawResult);
}

/// Enum to represent the different types of gear available in the application
#[derive(Clone, PartialEq)]
pub enum GearType {
    TrackEditor,    // Single instrument track editor
    ScoreEditor,    // Multi-instrument score editor
    Mixer,          // (Future) Mixer
    // Add more gear types as needed
}