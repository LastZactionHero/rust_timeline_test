mod track_editor;  // Module for track editing
mod score_editor;  // Module for score editing (all instruments)

use crate::draw_components::{DrawComponent, ViewportDrawResult};
use crate::events::InputEvent;

pub use track_editor::TrackEditorGear;  // Export the track editor gear
pub use score_editor::ScoreEditorGear;  // Export the score editor gear

use std::any::Any;

/// Trait that defines the interface for all gear types
pub trait Gear: Any {
    /// Returns a reference to the draw component for this gear
    fn get_draw_component(&self) -> Box<dyn DrawComponent>;
    
    /// Handle an input event, return true if event was handled
    fn handle_event(&mut self, event: &InputEvent) -> bool;
    
    /// Get the name of this gear for display purposes
    fn name(&self) -> &'static str;
    
    /// Set the viewport draw result from the last render
    fn set_viewport_draw_result(&mut self, result: ViewportDrawResult);
    
    /// Allow downcasting the gear to its concrete type
    fn as_any(&self) -> &dyn Any;
    
    /// Allow downcasting the gear to its concrete type (mutable version)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Enum to represent the different types of gear available in the application
#[derive(Clone, PartialEq)]
pub enum GearType {
    TrackEditor,    // Single instrument track editor
    ScoreEditor,    // Multi-instrument score editor
    Mixer,          // (Future) Mixer
    // Add more gear types as needed
}