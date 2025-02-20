use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use chrono::Local;

use crate::score::Score;
use crate::pitch::Tone;

pub struct SongFile {
    current_path: Option<PathBuf>,
}

impl SongFile {
    pub fn new() -> Self {
        SongFile {
            current_path: None,
        }
    }

    fn generate_default_filename(&self) -> PathBuf {
        let date = Local::now().format("%Y%m%d");
        PathBuf::from(format!("song_{}.txt", date))
    }

    pub fn save(&mut self, score: &Score) -> io::Result<()> {
        let path = self.current_path.clone()
            .unwrap_or_else(|| self.generate_default_filename());
        
        let mut file = File::create(&path)?;
        
        // Write BPM
        writeln!(file, "BPM: {}", score.bpm)?;
        
        // Write notes
        let mut sorted_times: Vec<_> = score.notes.keys().collect();
        sorted_times.sort();
        
        for &time in sorted_times {
            if let Some(notes) = score.notes.get(&time) {
                let mut note_strs = Vec::new();
                
                for note in notes {
                    let tone_str = match note.pitch.tone {
                        Tone::C => "C",
                        Tone::Cs => "Cs",
                        Tone::D => "D",
                        Tone::Ds => "Ds",
                        Tone::E => "E",
                        Tone::F => "F",
                        Tone::Fs => "Fs",
                        Tone::G => "G",
                        Tone::Gs => "Gs",
                        Tone::A => "A",
                        Tone::As => "As",
                        Tone::B => "B",
                    };
                    
                    note_strs.push(format!("{}{}-{}", 
                        tone_str,
                        note.pitch.octave,
                        note.duration_b32
                    ));
                }
                
                writeln!(file, "{}: {}", time, note_strs.join(" "))?;
            }
        }
        
        self.current_path = Some(path);
        Ok(())
    }
} 