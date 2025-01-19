use crate::score::Score;
use crate::song::create_song;
use crossterm::{
    cursor::{self},
    event::{poll, read, Event, KeyCode},
    style::{self},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

use crate::draw_components::{
    self, score_draw_component::ScoreDrawComponent, BoxDrawComponent, DrawComponent, NullComponent,
    Position, Window,
};
use std::io::{self};

pub struct AppState {
    score: &'static Score,
}

impl AppState {
    pub fn new(score: &'static Score) -> AppState {
        AppState { score }
    }

    pub fn run(&self) -> io::Result<()> {
        println!("Hello from App State!");
        self.draw()?;
        Ok(())
    }

    fn draw(&self) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        // let width = 100;
        println!("{}", width);
        let mut buffer = vec![vec![' '; width as usize]; height as usize];

        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::All))?;

        let base_component = Window::new(vec![Box::new(BoxDrawComponent::new(Box::new(
            draw_components::VSplitDrawComponent::new(
                Box::new(ScoreDrawComponent::new(self.score)),
                Box::new(draw_components::FillComponent { value: '0' }),
            ),
        )))]);
        let position = Position {
            x: 0,
            y: 0,
            w: width as usize,
            h: height as usize,
        };
        base_component.draw(&mut buffer, &position);

        for y in 0..height {
            let row: String = buffer[y as usize].clone().into_iter().collect();
            stdout
                .queue(cursor::MoveTo(0, y))?
                .queue(style::Print(row))?;
        }

        Ok(())
    }
}
