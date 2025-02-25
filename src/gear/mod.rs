mod track_editor;  // New module for track editing

use crate::draw_components::{DrawComponent, DrawResult, Position, ViewportDrawResult};
use crate::events::InputEvent;

pub use track_editor::TrackEditorGear;  // Export the new track editor gear

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
#[derive(Clone)]
pub enum GearType {
    TrackEditor,    // Renamed from ScoreEditor
    Mixer,
    // Add more gear types as needed
}