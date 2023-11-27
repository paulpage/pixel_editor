use super::app::{self, Key, Color, Rect, Vec2, Font};

#[derive(Default)]
enum SizeKind {
    #[default]
    Pixels,
    PercentOfParent,
    TextContent,
    ChildrenSum,
}
#[derive(Default)]
struct AxisSize {
    kind: SizeKind,
    value: f32,
    strictness: f32,
}
#[derive(Default)]
struct Size {
    x: AxisSize,
    y: AxisSize,
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
    size: Size,
    flags: u64,
    layout: Layout,

    // Computed values
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
    Rows(i32),
    Columns(i32),
}

#[derive(Default)]
pub struct Ui {
    widgets: Vec<Widget>,
    root: usize,
    current_id: usize,
    font: Option<Font>,
    next_floating_window_pos: Vec2,
    font_size: f32,
    color_text: Color,
    color_background: Color,
}

impl Ui {
    pub fn new() -> Self {

        let data = std::fs::read("/usr/share/fonts/TTF/DejaVuSans.ttf").unwrap();

        let mut ui = Self {
            next_floating_window_pos: Vec2::new(20.0, 40.0),
            font: Some(app::load_ttf_font_from_bytes(&data).unwrap()),
            font_size: 20.0,
            color_text: app::WHITE,
            ..Default::default()
        };

        ui.widgets.push(Widget {
            name: "ROOT".to_string(),
            size: Size {
                x: AxisSize {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                y: AxisSize {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            },
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
        let (new_id, interaction) = self.check_widget(Widget {
            name: name.to_string(),
            size: Size {
                x: AxisSize {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
                y: AxisSize {
                    kind: SizeKind::PercentOfParent,
                    value: 100.0,
                    strictness: 0.0,
                },
            },
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
            size: Size {
                x: AxisSize {
                    kind: SizeKind::TextContent,
                    value: 0.0,
                    strictness: 1.0,
                },
                y: AxisSize {
                    kind: SizeKind::TextContent,
                    value: 0.0,
                    strictness: 1.0,
                },
            },
            flags: WidgetFlags::Clickable | WidgetFlags::DrawBorder | WidgetFlags::DrawText,
            ..Default::default()
        });
        interaction
    }

    pub fn update_node(&mut self, id: usize, level: usize) {
        let offset = Vec2::new(0.0, 0.0);

        for i in 0..self.widgets[id].children.len() {
            let child_id = self.widgets[id].children[i];

            let text_dimensions = self.measure_text(&self.widgets[child_id].name);
            self.widgets[child_id].computed_rect = Rect {
                x: offset.x,
                y: offset.y - text_dimensions.offset_y,
                w: text_dimensions.width,
                h: text_dimensions.height,
            };

            println!("{}Node: {} {:?}", " ".repeat(level), self.widgets[child_id].name, self.widgets[child_id].computed_rect);

            self.update_node(child_id, level + 1);
        }
    }

    pub fn update(&mut self) {
        println!("========================================");
        self.update_node(self.root, 0);
        println!("Node count: {}", self.widgets.len());
    }
}
