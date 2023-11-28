use super::app::{self, Key, Color, Rect, Vec2, Font};

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

#[allow(non_upper_case_globals, non_snake_case)]
pub mod WidgetFlags {
    pub const DrawText: u64 = 0x01;
    pub const Clickable: u64 = 0x02;
    pub const DrawBorder: u64 = 0x04;
    // 0x08
    // 0x10
    // 0x20
    // 0x40
    // 0x80
    // 0x100
    // 0x200
    // ...
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

    // Computed values
    computed_size: [f32; 2],
    computed_rect: Rect,
    // computed_rel_position: Vec2,
    // computed_size: Vec2,
    rect: Rect,
}

#[derive(Default)]
pub struct Interaction {
    pub clicked: bool,
}

#[derive(Default)]
pub enum Layout {
    #[default]
    Null,
    Floating,
    Row,
    Column,
}

#[derive(Default)]
pub struct Ui {
    widgets: Vec<Widget>,
    root: usize,
    current_id: usize,
    font: Option<Font>,
    next_floating_window_pos: Vec2,
    font_size: f32,
    border_size: f32,
    color_background: Color,
    color_border: Color,
    color_text: Color,
}

impl Ui {
    pub fn new() -> Self {

        let data = std::fs::read("/usr/share/fonts/TTF/DejaVuSans.ttf").unwrap();

        let mut ui = Self {
            next_floating_window_pos: Vec2::new(20.0, 40.0),
            font: Some(app::load_ttf_font_from_bytes(&data).unwrap()),
            font_size: 20.0,
            border_size: 2.0,
            color_background: app::GRAY,
            color_border: app::GREEN,
            color_text: app::WHITE,
            ..Default::default()
        };

        ui.widgets.push(Widget {
            name: "ROOT".to_string(),
            size: [
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            ],
            layout: Layout::Floating,
            ..Default::default()
        });

        ui
    }

    fn draw_text(&self, text: &str, x: f32, y: f32) {
        app::draw_text_ex(text, x, y, app::TextParams {
            font_size: self.font_size as u16,
            font_scale: 1.0,
            font: self.font.as_ref(),
            color: self.color_text,
            ..Default::default()
        });
    }

    fn measure_text(&self, text: &str) -> app::TextDimensions {
        app::measure_text(text, self.font.as_ref(), self.font_size as u16, 1.0)
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
            let (mouse_x, mouse_y) = app::mouse_position();
            if self.widgets[id].rect.contains(Vec2::new(mouse_x, mouse_y)) && app::is_mouse_left_pressed() {
                interaction.clicked = true;
            }
        } else {
            let mut widget = widget;
            widget.id = self.widgets.len();
            widget.parent = self.current_id;
            self.widgets[self.current_id].children.push(widget.id);
            target_id = Some(widget.id);
            self.widgets.push(widget);
        }

        (target_id.unwrap(), interaction)
    }

    pub fn push_layout(&mut self, name: &str, layout: Layout) -> Interaction {
        let size = match layout {
            Layout::Null => [
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            ],
            Layout::Floating => [
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            ],
            Layout::Row => [
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 1.0,
                },
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            ],
            Layout::Column => [
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 1.0,
                },
            ],
        };
        let (new_id, interaction) = self.check_widget(Widget {
            name: name.to_string(),
            size: size,
            layout: layout,
            ..Default::default()
        });
        self.current_id = new_id;
        return interaction;
    }

    pub fn pop_layout(&mut self) {
        self.current_id = self.widgets[self.current_id].parent;
    }

    pub fn floating_window(&mut self, name: &str) -> Interaction {
        let (_, interaction) = self.check_widget(Widget {
            // TODO
            ..Default::default()
        });
        interaction
    }

    pub fn button(&mut self, name: &str) -> Interaction {
        let (_, interaction) = self.check_widget(Widget {
            id: self.widgets.len(),
            name: name.to_string(),
            size: [
                Size {
                    kind: SizeKind::TextContent,
                    value: 0.0,
                    strictness: 1.0,
                },
                Size {
                    kind: SizeKind::TextContent,
                    value: 0.0,
                    strictness: 1.0,
                },
            ],
            flags: WidgetFlags::Clickable | WidgetFlags::DrawBorder | WidgetFlags::DrawText,
            ..Default::default()
        });
        interaction
    }

    pub fn spacer(&mut self, name: &str) -> Interaction {
        let (_, interaction) = self.check_widget(Widget {
            id: self.widgets.len(),
            name: name.to_string(),
            size: [
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                Size {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            ],
            flags: 0,
            ..Default::default()
        });
        interaction
    }

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
        println!("{}parent_dep - {} - {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].computed_size);

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
                        println!("have a child for {} j={} : {} <? {}", self.widgets[id].name, j, self.widgets[id].computed_size[j], self.widgets[child_id].computed_size[j]);
                        if self.widgets[id].computed_size[j] < self.widgets[child_id].computed_size[j] {
                            println!("actually updating child");
                            self.widgets[id].computed_size[j] = self.widgets[child_id].computed_size[j];
                        }
                    }
                    _ => {}
                }
            }

        }
        println!("{}child_dep: {} {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].computed_size);
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
            println!("violations total for {} j={}: {}", self.widgets[id].name, j, total);
            if total > self.widgets[id].computed_size[j] {
                let difference = total - self.widgets[id].computed_size[j];
                println!("violations UH-OHHHHHH  for {} j={}: {} over", self.widgets[id].name, j, difference);
                let mut available = 0.0;
                for i in 0..self.widgets[id].children.len() {
                    let child_id = self.widgets[id].children[i];
                    available += self.widgets[child_id].computed_size[j] * (1.0 - self.widgets[child_id].size[j].strictness);
                }

                let shrink_multiplier = difference / available;
                if shrink_multiplier > 1.0 {
                    println!("WARNING: Not enough to shrink");
                } else {
                    for i in 0..self.widgets[id].children.len() {
                        let child_id = self.widgets[id].children[i];
                        let available = self.widgets[child_id].computed_size[j] * (1.0 - self.widgets[child_id].size[j].strictness);
                        println!("FIXXXX {} for {} j={} shrink {} to {}", self.widgets[child_id].name, self.widgets[id].name, j, self.widgets[child_id].computed_size[j], self.widgets[child_id].computed_size[j] - available * shrink_multiplier);
                        self.widgets[child_id].computed_size[j] -= available * shrink_multiplier;
                    }
                }

            }
        }

        println!("{}violations: {} {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].computed_size);
    }

    fn calc_positions(&mut self, id: usize, level: usize, pos: Vec2) {
        let mut child_pos = pos;
        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.calc_positions(child_id, level + 1, child_pos);
            match self.widgets[id].layout {
                Layout::Null => {},
                Layout::Floating => {},
                Layout::Row => {
                    child_pos.x += self.widgets[child_id].computed_size[0];
                },
                Layout::Column => {
                    child_pos.y += self.widgets[child_id].computed_size[1];
                },
            }
        }

        let parent = self.widgets[id].parent;
        self.widgets[id].rect = Rect {
            x: pos.x,
            y: pos.y,
            w: self.widgets[id].computed_size[0],
            h: self.widgets[id].computed_size[1],
        };
    }

    pub fn draw_node(&mut self, id: usize, level: usize) {
        println!("{}draw {}: {:?}", " ".repeat(level), self.widgets[id].name, self.widgets[id].rect);

        let flags = self.widgets[id].flags;
        if flags & WidgetFlags::DrawBorder != 0 {
            app::draw_rect(self.widgets[id].rect, self.color_border);
            let inside_rect = Rect {
                x: self.widgets[id].rect.x + self.border_size,
                y: self.widgets[id].rect.y + self.border_size,
                w: self.widgets[id].rect.w - self.border_size * 2.0,
                h: self.widgets[id].rect.h - self.border_size * 2.0,
            };
            app::draw_rect(inside_rect, self.color_background);
        } else {
            app::draw_rect(self.widgets[id].rect, self.color_background);
        }

        if flags & WidgetFlags::DrawText != 0 {
            self.draw_text(&self.widgets[id].name, self.widgets[id].rect.x, self.widgets[id].rect.y);
        }

        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];
            self.draw_node(child_id, level + 1);
        }
    }

    pub fn update(&mut self) {
        println!("========================================");
        self.widgets[self.root].size = [
            Size {
                kind: SizeKind::Pixels,
                value: app::screen_width(),
                strictness: 1.0,
            },
            Size {
                kind: SizeKind::Pixels,
                value: app::screen_height(),
                strictness: 1.0,
            }
        ];

        for i in 0..self.widgets.len() {
            for j in 0..2 {
                match self.widgets[i].size[j].kind {
                    SizeKind::Pixels => {
                        self.widgets[i].computed_size[j] = self.widgets[i].size[j].value;
                    }
                    SizeKind::TextContent => {
                        let text_dimensions = self.measure_text(&self.widgets[i].name);
                        let text_size = [text_dimensions.width, text_dimensions.height];
                        self.widgets[i].computed_size[j] = text_size[j];
                    }
                    _ => {}
                }
            }
        }
        
        self.calc_parent_dependent(self.root, 0);
        self.calc_child_dependent(self.root, 0);

        self.calc_violations(self.root, 0);
        self.calc_positions(self.root, 0, Vec2::new(0.0, 0.0));

        // self.update_node(self.root, 0, Vec2::new(0.0, 0.0));
        println!("Node count: {}", self.widgets.len());

        println!("-------------------------------");
        self.draw_node(self.root, 0);
    }
}
