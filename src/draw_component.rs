const BOX_TOP_LEFT: char = '╔';
const BOX_TOP_RIGHT: char = '╗';
const BOX_BOTTOM_LEFT: char = '╚';
const BOX_BOTTOM_RIGHT: char = '╝';
const BOX_HORIZONTAL: char = '═';
const BOX_VERTICAL: char = '║';

pub struct Position {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

pub trait DrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, position: &Position);

    fn wb(&self, buffer: &mut Vec<Vec<char>>, position: &Position, point: &Point, value: char) {
        buffer[position.y + point.y][position.x + point.x] = value;
    }
}

pub struct Window {
    components: Vec<Box<dyn DrawComponent>>,
}

impl Window {
    pub fn new(components: Vec<Box<dyn DrawComponent>>) -> Window {
        Window { components }
    }
}

impl DrawComponent for Window {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, position: &Position) {
        for component in &self.components {
            component.draw(buffer, &position);
        }
    }
}

pub struct BoxDrawComponent {
    top_component: Box<dyn DrawComponent>,
    bottom_component: Box<dyn DrawComponent>,
}

impl BoxDrawComponent {
    pub fn new(
        top_component: Box<dyn DrawComponent>,
        bottom_component: Box<dyn DrawComponent>,
    ) -> BoxDrawComponent {
        BoxDrawComponent {
            top_component,
            bottom_component,
        }
    }
}

impl DrawComponent for BoxDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, position: &Position) {
        for x in position.x + 1..position.x + position.w - 1 {
            self.wb(buffer, position, &Point::new(x, 0), BOX_HORIZONTAL);
            self.wb(
                buffer,
                position,
                &Point::new(x, position.h - 1),
                BOX_HORIZONTAL,
            );
        }
        for y in position.y + 1..position.y + position.h - 1 {
            buffer[y][0] = BOX_VERTICAL;
            buffer[y][position.x + position.w - 1] = BOX_VERTICAL;
        }
        buffer[position.y][position.x] = BOX_TOP_LEFT;
        buffer[position.y][position.x + position.w - 1] = BOX_TOP_RIGHT;
        buffer[position.y + position.h - 1][position.x] = BOX_BOTTOM_LEFT;
        buffer[position.y + position.h - 1][position.x + position.w - 1] = BOX_BOTTOM_RIGHT;
    }
}

pub struct NullComponent {}

impl DrawComponent for NullComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, position: &Position) {}
}
