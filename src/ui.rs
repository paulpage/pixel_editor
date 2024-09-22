use super::app::{self as g, Key, Color, Rect, Vec2, Font};

// ============================================================

#[macro_export]
macro_rules! temp_style {
    ($ui:expr, $($field:ident: $value: expr),* $(,)?) => {{
        let mut style = $ui.get_current_style();
        $(
            style.$field = $value;
        )*
        $ui.set_temp_style(style);
    }};
}

#[macro_export]
macro_rules! push_style {
    ($ui:expr, $($field:ident: $value: expr),* $(,)?) => {{
        let mut style = $ui.get_current_style();
        $(
            style.$field = $value;
        )*
        $ui.push_style(style);
    }};
}

// ============================================================

#[derive(Default)]
enum SizeKind {
    #[default]
    Pixels,
    PercentOfParent,
    TextContent,
    ChildrenSum,
    ChildrenMax,
}
#[derive(Default)]
struct Size {
    kind: SizeKind,
    value: f32,
    strictness: f32,
}

impl Size {
    pub fn new(kind: SizeKind, value: f32, strictness: f32) -> Self {
        Self {
            kind,
            value,
            strictness,
        }
    }
}

pub mod WidgetFlags {
    pub const DRAW_TEXT: u64 = 1 << 0;
    pub const CLICKABLE: u64 = 1 << 1;
    pub const DRAW_BORDER: u64 = 1 << 2;
    pub const MOVABLE: u64 = 1 << 3;
    pub const INVISIBLE: u64 = 1 << 4;
}

#[derive(Default)]
struct Widget {
    // Tree
    id: usize,
    parent: usize,
    children: Vec<usize>,

    // Basic info
    name: String,
    size: [Size; 2],
    flags: u64,
    layout: Layout,
    requested_pos: Vec2,
    style: StyleInfo,

    // State
    dragging: bool,
    hovered: bool,

    // Computed values
    computed_size: [f32; 2],
    computed_rect: Rect,
    rect: Rect,
}

#[derive(Default)]
pub struct Interaction {
    pub clicked: bool,
    pub hovered: bool,
}

#[derive(Default)]
pub enum Layout {
    #[default]
    Null,
    Floating,
    Horizontal,
    Vertical,
    ToolRow,
    ToolColumn,
}

#[derive(Default, Clone)]
pub struct StyleInfo {
    pub font: Option<Font>,
    pub font_size: f32,
    pub border_size: f32,
    pub padding: f32,
    pub color_background: Color,
    pub color_border: Color,
    pub color_text: Color,
}

#[derive(Default)]
pub struct Ui {
    // Tree
    windows: Vec<Window>,
    current_id: usize,

    // Style
    style: StyleInfo,

    // State
    next_floating_window_pos: Vec2,
    mouse_intercepted: bool,
}

#[derive(Default)]
pub struct Window {
    pub name: String,
    pub rect: Rect,

    // Style
    styles: Vec<StyleInfo>,
    temp_style_info: Option<StyleInfo>,

    widgets: Vec<Widget>,
    current_id: usize,

    mouse_intercepted: bool,
    zindex: usize,
}

fn measure_text(text: &str, style: &StyleInfo) -> g::TextDimensions {
    g::measure_text(text, style.font.as_ref(), style.font_size as u16, 1.0)
}


fn draw_text(text: &str, x: f32, y: f32, style: &StyleInfo) {
    g::draw_text_ex(text, x, y + style.font_size as f32, g::TextParams {
        font_size: style.font_size as u16,
        font_scale: 1.0,
        font: style.font.as_ref(),
        color: style.color_text,
        ..Default::default()
    });
}

impl Window {

    fn calc_parent_dependent(&mut self, id: usize, level: usize) {
        for i in 0..2 {
            match self.widgets[id].size[i].kind {
                SizeKind::PercentOfParent => {
                    let parent = self.widgets[id].parent;
                    let percent = self.widgets[id].size[i].value / 100.0;
                    self.widgets[id].computed_size[i] = self.widgets[parent].computed_size[i] * percent;
                }
                _ => {}
            }
        }
        // println!("{}parent_dep - {} - {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].computed_size);

        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.calc_parent_dependent(child_id, level + 1);
        }
    }

    fn calc_child_dependent(&mut self, id: usize, level: usize) {
        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.calc_child_dependent(child_id, level + 1);

            for j in 0..2 {
                match self.widgets[id].size[j].kind {
                    SizeKind::ChildrenSum => {
                        self.widgets[id].computed_size[j] += self.widgets[child_id].computed_size[j];
                    }
                    SizeKind::ChildrenMax => {
                        // println!("have a child for {} j={} : {} <? {}", self.widgets[id].name, j, self.widgets[id].computed_size[j], self.widgets[child_id].computed_size[j]);
                        if self.widgets[id].computed_size[j] < self.widgets[child_id].computed_size[j] {
                            // println!("actually updating child");
                            self.widgets[id].computed_size[j] = self.widgets[child_id].computed_size[j];
                        }
                    }
                    _ => {}
                }
            }

        }
        // println!("{}child_dep: {} {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].computed_size);
    }

    fn calc_violations(&mut self, id: usize, level: usize) {

        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.calc_violations(child_id, level + 1);
        }

        for j in 0..2 {
            let mut total = 0.0;
            for i in 0..self.widgets[id].children.len() {
                let child_id = self.widgets[id].children[i];
                total += self.widgets[child_id].computed_size[j];
            }
            // println!("violations total for {} j={}: {}", self.widgets[id].name, j, total);
            if total > self.widgets[id].computed_size[j] {
                let difference = total - self.widgets[id].computed_size[j];
                // println!("violations UH-OHHHHHH  for {} j={}: {} over", self.widgets[id].name, j, difference);
                let mut available = 0.0;
                for i in 0..self.widgets[id].children.len() {
                    let child_id = self.widgets[id].children[i];
                    available += self.widgets[child_id].computed_size[j] * (1.0 - self.widgets[child_id].size[j].strictness);
                }

                let shrink_multiplier = difference / available;
                if shrink_multiplier > 1.0 {
                    for i in 0..self.widgets[id].children.len() {
                        let child_id = self.widgets[id].children[i];
                        // TODO figure this out
                        // println!("WARNING: Not enough to shrink {} children j={} child_total={} self_total={} {}/{} > 1.0", self.widgets[id].name, j, total, self.widgets[id].computed_size[j], difference, available);
                    }
                } else {
                    for i in 0..self.widgets[id].children.len() {
                        let child_id = self.widgets[id].children[i];
                        let available = self.widgets[child_id].computed_size[j] * (1.0 - self.widgets[child_id].size[j].strictness);
                        // println!("FIXXXX {} for {} j={} shrink {} to {}", self.widgets[child_id].name, self.widgets[id].name, j, self.widgets[child_id].computed_size[j], self.widgets[child_id].computed_size[j] - available * shrink_multiplier);
                        self.widgets[child_id].computed_size[j] -= available * shrink_multiplier;
                    }
                }

            }
        }

        // println!("{}violations: {} {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].computed_size);
    }

    fn calc_positions(&mut self, id: usize, level: usize, pos: Vec2) {
        let mut child_pos = pos;
        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.calc_positions(child_id, level + 1, child_pos);
            match self.widgets[id].layout {
                Layout::Null => {},
                Layout::Floating => {
                    child_pos = self.widgets[child_id].requested_pos;
                },
                Layout::Horizontal => {
                    child_pos.x += self.widgets[child_id].computed_size[0];
                },
                Layout::Vertical => {
                    child_pos.y += self.widgets[child_id].computed_size[1];
                },
                Layout::ToolRow => {
                    child_pos.x += self.widgets[child_id].computed_size[0];
                },
                Layout::ToolColumn => {
                    child_pos.y += self.widgets[child_id].computed_size[1];
                },
            }
        }

        let parent = self.widgets[id].parent;
        self.widgets[id].rect = Rect {
            x: self.rect.x + pos.x,
            y: self.rect.y + pos.y,
            w: self.widgets[id].computed_size[0],
            h: self.widgets[id].computed_size[1],
        };
    }

    pub fn draw_node(&mut self, id: usize, level: usize, style: &StyleInfo) {

        let style = self.widgets[id].style.clone();

        let flags = self.widgets[id].flags;

        // println!("{}draw {}: {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].rect);
        // println!("I am {} and my children are {:?}", self.widgets[id].name, self.widgets[id].children);

        if flags & WidgetFlags::INVISIBLE == 0 {

            let color = if self.widgets[id].hovered && (flags & WidgetFlags::CLICKABLE != 0) {
                Color::new(0.5, 0.5, 0.5, 1.0)
            } else {
                style.color_background
            };

            if flags & WidgetFlags::DRAW_BORDER != 0 {
                g::draw_rect(self.widgets[id].rect, style.color_border);
                let inside_rect = Rect {
                    x: self.widgets[id].rect.x + style.border_size,
                    y: self.widgets[id].rect.y + style.border_size,
                    w: self.widgets[id].rect.w - style.border_size * 2.0,
                    h: self.widgets[id].rect.h - style.border_size * 2.0,
                };
                g::draw_rect(inside_rect, color);
            } else {
                g::draw_rect(self.widgets[id].rect, color);
            }

            if flags & WidgetFlags::DRAW_TEXT != 0 {
                draw_text(&self.widgets[id].name, self.widgets[id].rect.x + style.padding, self.widgets[id].rect.y + style.padding, &style);
            }
        }

        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.draw_node(child_id, level + 1, &style);
        }
    }

    pub fn calc_input(&mut self, id: usize, level: usize, mouse_intercepted: bool) {

        self.widgets[id].hovered = false;

        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.calc_input(child_id, level + 1, mouse_intercepted);
        }

        let (mouse_x, mouse_y) = g::mouse_position();

        // println!("{} widget rect {:?} window rect {:?} mouse {}, {}", self.widgets[id].name, self.widgets[id].rect, self.rect, mouse_x, mouse_y);
        let rect = Rect {
            x: self.widgets[id].rect.x,
            y: self.widgets[id].rect.y,
            w: self.widgets[id].rect.w,
            h: self.widgets[id].rect.h,
        };
        // if !self.mouse_intercepted && self.widgets[id].rect.contains(Vec2::new(self.rect.x + mouse_x, self.rect.y + mouse_y)) {
        if !(self.mouse_intercepted || mouse_intercepted) && rect.contains(Vec2::new(mouse_x, mouse_y)) {
            self.widgets[id].hovered = true;
            self.mouse_intercepted = true;
            // println!("INTERCEPTED: {} rect {:?}", self.widgets[id].name, self.widgets[id].rect);
        }
    }

    fn check_widget(&mut self, widget: Widget) -> (usize, Interaction) {
        let mut interaction = Interaction::default();

        let mut target_id = None;
        for widget_id in &self.widgets[self.current_id].children {
            if self.widgets[*widget_id].name == widget.name {
                target_id = Some(*widget_id);
            }
        }

        if let Some(id) = target_id {
            let (mouse_x, mouse_y) = g::mouse_position();

            if self.widgets[id].rect.contains(Vec2::new(mouse_x, mouse_y)) {
                interaction.hovered = true;
            }

            if self.widgets[id].rect.contains(Vec2::new(mouse_x, mouse_y)) && g::is_mouse_left_pressed() {
                interaction.clicked = true;
                if self.widgets[id].flags & WidgetFlags::MOVABLE != 0 {
                    self.widgets[id].dragging = true;
                }
            }

            if !g::is_mouse_left_down() {
                self.widgets[id].dragging = false;
            }

            if widget.dragging {
                println!("dragging");
            }

        } else {
            let mut widget = widget;
            widget.id = self.widgets.len();
            widget.parent = self.current_id;
            self.widgets[self.current_id].children.push(widget.id);
            target_id = Some(widget.id);
            self.widgets.push(widget);
        }

        self.widgets[target_id.unwrap()].style = if let Some(style) = self.temp_style_info.clone() {
            let style = style.clone();
            self.temp_style_info = None;
            style
        } else {
            self.styles[self.styles.len() - 1].clone()
        };

        (target_id.unwrap(), interaction)
    }
}

impl Ui {
    pub fn new() -> Self {

        // let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf").unwrap();
        let data = include_bytes!("../data/fonts/font.ttf");


        let style = StyleInfo {
            font: Some(g::load_ttf_font_from_bytes(data).unwrap()),
            font_size: 20.0,
            border_size: 2.0,
            padding: 5.0,
            color_background: g::DARKGRAY,
            color_border: g::GREEN,
            color_text: g::WHITE,
            ..Default::default()
        };

        let mut ui = Self {
            next_floating_window_pos: Vec2::new(20.0, 40.0),
            style: style.clone(),
            ..Default::default()
        };

        let window = Window {
            name: "FIRST_ROOT_WINDOW".to_string(),
            styles: vec![style],
            ..Default::default()
        };
        ui.windows.push(window);
        ui.windows[0].widgets.push(Widget {
            name: "FIRST_ROOT_WIDGET".to_string(),
            size: [
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
            ],
            layout: Layout::Floating,
            flags: WidgetFlags::INVISIBLE,
            ..Default::default()
        });

        ui
    }

    // ============================================================

    pub fn get_current_style(&self) -> StyleInfo {
        let window = &self.windows[self.current_id];
        window.styles[window.styles.len() - 1].clone()
    }

    pub fn push_style(&mut self, style: StyleInfo) {
        self.windows[self.current_id].styles.push(style);
    }

    pub fn pop_style(&mut self) {
        self.windows[self.current_id].styles.pop();
    }

    pub fn set_temp_style(&mut self, style: StyleInfo) {
        self.windows[self.current_id].temp_style_info = Some(style);
    }

    // ============================================================

    pub fn push_layout(&mut self, name: &str, layout: Layout) -> Interaction {
        let w = self.current_id;
        let size = match layout {
            Layout::Null => [
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
            ],
            Layout::Floating => [
                Size::new(SizeKind::ChildrenSum, 0.0, 0.0),
                Size::new(SizeKind::ChildrenSum, 0.0, 0.0),
            ],
            Layout::Horizontal => [
                Size::new(SizeKind::PercentOfParent, 100.0, 1.0),
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
            ],
            Layout::Vertical => [
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
                Size::new(SizeKind::PercentOfParent, 100.0, 1.0),
            ],
            Layout::ToolRow => [
                Size::new(SizeKind::PercentOfParent, 100.0, 1.0),
                Size::new(SizeKind::ChildrenMax, 0.0, 1.0),
            ],
            Layout::ToolColumn => [
                Size::new(SizeKind::ChildrenMax, 0.0, 1.0),
                Size::new(SizeKind::PercentOfParent, 100.0, 1.0),
            ]
        };
        let flags = match layout {
            Layout::Null => WidgetFlags::INVISIBLE,
            Layout::Floating => WidgetFlags::MOVABLE | WidgetFlags::INVISIBLE,
            Layout::Vertical => WidgetFlags::INVISIBLE,
            Layout::Horizontal => WidgetFlags::INVISIBLE,
            _ => 0,
        };
        let (new_id, interaction) = self.windows[w].check_widget(Widget {
            name: name.to_string(),
            size,
            layout,
            flags,
            ..Default::default()
        });
        self.windows[w].current_id = new_id;
        return interaction;
    }

    pub fn pop_layout(&mut self) {
        let w = self.current_id;
        self.windows[w].current_id = self.windows[w].widgets[self.windows[w].current_id].parent;
    }

    pub fn button(&mut self, name: &str) -> Interaction {
        let w = self.current_id;
        let id = self.windows[w].widgets.len();
        let (_, interaction) = self.windows[w].check_widget(Widget {
            id,
            name: name.to_string(),
            size: [
                Size::new(SizeKind::TextContent, 0.0, 1.0),
                Size::new(SizeKind::TextContent, 0.0, 1.0),
            ],
            flags: WidgetFlags::CLICKABLE | WidgetFlags::DRAW_BORDER | WidgetFlags::DRAW_TEXT,
            ..Default::default()
        });
        interaction
    }

    pub fn spacer(&mut self, name: &str) -> Interaction {
        let w = self.current_id;
        let id = self.windows[w].widgets.len();
        let (_, interaction) = self.windows[w].check_widget(Widget {
            id,
            name: name.to_string(),
            size: [
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
                Size::new(SizeKind::PercentOfParent, 100.0, 0.0),
            ],
            flags: WidgetFlags::INVISIBLE,
            ..Default::default()
        });
        interaction
    }

    fn check_window(&mut self, window: Window) {

        let rect = window.rect;

        let mut target_id = None;
        for w in 0..self.windows.len() {
            if self.windows[w].name == window.name {
                target_id = Some(w);
            }
        }

        if let Some(id) = target_id {
            self.current_id = id;
        } else {
            println!("NEW WINDOW");
            self.current_id = self.windows.len();
            self.windows.push(window);
            self.windows[self.current_id].widgets.push(Widget {
                name: "ANOTHER_ROOT_WIDGET".to_string(),
                size: [
                    Size::new(SizeKind::Pixels, rect.w, 1.0),
                    Size::new(SizeKind::Pixels, rect.h, 1.0),
                ],
                layout: Layout::Floating,
                flags: WidgetFlags::INVISIBLE,
                ..Default::default()
            });
        }
    }

    pub fn push_window(&mut self, name: &str, rect: Rect) {
        self.check_window(Window {
            name: name.to_string(),
            rect,
            styles: vec![self.style.clone()],
            // style: self.style.clone(),
            ..Default::default()
        });
    }

    pub fn update(&mut self) {

        self.current_id = 0;
        self.mouse_intercepted = false;

        // println!("========================================");
        self.windows[0].rect = Rect::new(0.0, 0.0, g::screen_width(), g::screen_height());

        for w in 0..self.windows.len() {

            let rect = self.windows[w].rect;
            self.windows[w].widgets[0].size = [
                Size::new(SizeKind::Pixels, rect.w, 1.0),
                Size::new(SizeKind::Pixels, rect.h, 1.0),
            ];

            self.windows[w].mouse_intercepted = self.mouse_intercepted;
            self.windows[w].current_id = 0;

            // println!("---------------------------------------- drawing window {}", self.windows[w].name);

            for i in 0..self.windows[w].widgets.len() {
                for j in 0..2 {
                    match self.windows[w].widgets[i].size[j].kind {
                        SizeKind::Pixels => {
                            self.windows[w].widgets[i].computed_size[j] = self.windows[w].widgets[i].size[j].value;
                        }
                        SizeKind::TextContent => {
                            let text_dimensions = measure_text(&self.windows[w].widgets[i].name, &self.style);
                            let text_size = [text_dimensions.width + (self.style.padding + self.style.border_size) * 2.0, self.style.font_size as f32 + (self.style.padding + self.style.border_size) * 2.0];
                            self.windows[w].widgets[i].computed_size[j] = text_size[j];
                        }
                        _ => {}
                    }
                }
            }

            self.windows[w].calc_parent_dependent(0, 0);
            self.windows[w].calc_child_dependent(0, 0);

            self.windows[w].calc_violations(0, 0);
            self.windows[w].calc_positions(0, 0, Vec2::new(0.0, 0.0));
        }

        for w in (0..self.windows.len()).rev() {
            self.windows[w].calc_input(0, 0, self.mouse_intercepted);
            if self.windows[w].mouse_intercepted {
                self.mouse_intercepted = true;
            }
        }

        for w in 0..self.windows.len() {
            for i in 0..self.windows[w].widgets.len() {
                // println!("-------------------------------");
                self.windows[w].draw_node(0, 0, &mut self.style);
            }
        }
    }
}
