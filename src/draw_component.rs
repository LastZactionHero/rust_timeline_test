const BOX_TOP_LEFT: char = '╔';
const BOX_TOP_RIGHT: char = '╗';
const BOX_BOTTOM_LEFT: char = '╚';
const BOX_BOTTOM_RIGHT: char = '╝';
const BOX_HORIZONTAL: char = '═';
const BOX_VERTICAL: char = '║';
const BOX_RIGHT_DIVIDER: char = '╣';
const BOX_LEFT_DIVIDER: char = '╠';

pub struct Position {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Position {
    fn right(&self) -> usize {
        self.x + self.w - 1
    }

    fn bottom(&self) -> usize {
        self.y + self.h - 1
    }
}

pub trait DrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position);

    fn wb(&self, buffer: &mut Vec<Vec<char>>, pos: &Position, x: usize, y: usize, value: char) {
        buffer[pos.y + y][pos.x + x] = value;
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
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        for component in &self.components {
            component.draw(buffer, &pos);
        }
    }
}

pub struct BoxDrawComponent {
    component: Box<dyn DrawComponent>,
}

impl BoxDrawComponent {
    pub fn new(component: Box<dyn DrawComponent>) -> BoxDrawComponent {
        BoxDrawComponent { component }
    }
}

impl DrawComponent for BoxDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        for x in pos.x + 1..pos.right() {
            self.wb(buffer, pos, x, 0, BOX_HORIZONTAL);
            self.wb(buffer, pos, x, pos.h - 1, BOX_HORIZONTAL);
        }
        for y in pos.y + 1..pos.bottom() {
            buffer[y][0] = BOX_VERTICAL;
            buffer[y][pos.x + pos.w - 1] = BOX_VERTICAL;
        }
        self.wb(buffer, pos, pos.x, pos.y, BOX_TOP_LEFT);
        self.wb(buffer, pos, pos.right(), pos.y, BOX_TOP_RIGHT);
        self.wb(buffer, pos, pos.x, pos.bottom(), BOX_BOTTOM_LEFT);
        self.wb(buffer, pos, pos.right(), pos.bottom(), BOX_BOTTOM_RIGHT);

        self.component.draw(buffer, pos);
    }
}

pub struct VSplitDrawComponent {
    top_component: Box<dyn DrawComponent>,
    bottom_component: Box<dyn DrawComponent>,
}

impl VSplitDrawComponent {
    pub fn new(
        top_component: Box<dyn DrawComponent>,
        bottom_component: Box<dyn DrawComponent>,
    ) -> VSplitDrawComponent {
        VSplitDrawComponent {
            top_component,
            bottom_component,
        }
    }
}

impl DrawComponent for VSplitDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        let pos_top = Position {
            x: pos.x + 1,
            y: pos.y + 1,
            w: pos.w - 2,
            h: pos.h / 2,
        };
        let pos_bottom = Position {
            x: pos.x + 1,
            y: pos.y + pos.h / 2 + 2,
            w: pos.w - 2,
            h: pos.h / 2 - 2,
        };
        self.top_component.draw(buffer, &pos_top);
        self.bottom_component.draw(buffer, &pos_bottom);
        for x in 1..pos.w - 1 {
            self.wb(buffer, pos, x, pos.h / 2 + 1, '═');
        }
        self.wb(buffer, pos, 0, pos.h / 2 + 1, BOX_LEFT_DIVIDER);
        self.wb(buffer, pos, pos.w - 1, pos.h / 2 + 1, BOX_RIGHT_DIVIDER);
    }
}

pub struct NullComponent {}

impl DrawComponent for NullComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {}
}

pub struct FillComponent {
    pub value: char,
}

impl DrawComponent for FillComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) {
        for x in 0..pos.w {
            for y in 0..pos.h {
                self.wb(buffer, pos, x, y, self.value);
            }
        }
    }
}
