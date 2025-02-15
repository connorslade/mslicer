use bitflags::bitflags;
use egui::{Align, Layout, RichText, Sense, TextStyle, Ui, Vec2};
use markdown::{Block, Span};

const BODY_SIZE: f32 = 12.5;
const HEADING_SIZES: [f32; 6] = [18.0, 16.0, 14.0, 12.0, 10.0, 8.0];

pub struct CompiledMarkdown {
    nodes: Vec<Node>,
}

enum Node {
    Body(Vec<BodyNode>),
    Break,
}

enum BodyNode {
    Text {
        text: String,
        size: f32,
        flags: TextFlags,
    },
    Link {
        text: String,
        url: String,
    },
}

bitflags! {
    #[derive(Clone, Copy)]
    struct TextFlags: u8 {
        const HEADER = 1 << 0;
        const WEAK = 1 << 1;
        const BOLD = 1 << 2;
        const ITALIC = 1 << 3;
        const MONOSPACE = 1 << 4;
    }
}

impl TextFlags {
    pub fn apply(&self, mut text: RichText) -> RichText {
        if self.contains(TextFlags::HEADER) {
            text = text.heading();
        }

        if self.contains(TextFlags::WEAK) {
            text = text.weak();
        }

        if self.contains(TextFlags::BOLD) {
            text = text.strong();
        }

        if self.contains(TextFlags::ITALIC) {
            text = text.italics();
        }

        if self.contains(TextFlags::MONOSPACE) {
            text = text.monospace();
        }

        text
    }
}

impl CompiledMarkdown {
    pub fn compile(source: &str) -> Self {
        let mut nodes = Vec::new();

        for token in markdown::tokenize(source) {
            match token {
                Block::Header(span, level) => {
                    let mut flags = TextFlags::HEADER;

                    if level > 1 {
                        flags |= TextFlags::WEAK;
                    }

                    nodes.push(Node::Body(span_text(span, HEADING_SIZES[level - 1], flags)));
                    nodes.push(Node::Break);
                }
                Block::Paragraph(span) => {
                    nodes.push(Node::Body(span_text(span, BODY_SIZE, TextFlags::empty())));
                    nodes.push(Node::Break);
                }
                _ => {}
            }
        }

        Self { nodes }
    }

    pub fn render(&self, ui: &mut Ui) {
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
            Layout::left_to_right(Align::Max).with_main_wrap(true),
            |ui| {
                let row_height = ui.text_style_height(&TextStyle::Heading);
                ui.set_row_height(row_height);
                ui.spacing_mut().item_spacing.x = 0.0;

                for node in self.nodes.iter() {
                    match node {
                        Node::Body(body_nodes) => {
                            for node in body_nodes {
                                match node {
                                    BodyNode::Text { text, size, flags } => {
                                        ui.label(flags.apply(RichText::new(text).size(*size)));
                                    }
                                    BodyNode::Link { text, url } => {
                                        ui.hyperlink_to(text, url).on_hover_text(url);
                                    }
                                }
                            }
                        }
                        Node::Break => {
                            ui.allocate_exact_size(Vec2::new(0.0, row_height), Sense::hover());
                            ui.end_row();
                            ui.set_row_height(row_height);
                        }
                    }
                }
            },
        );
    }
}

fn span_text(span: Vec<Span>, size: f32, flags: TextFlags) -> Vec<BodyNode> {
    fn span_text_inner(out: &mut Vec<BodyNode>, span: &[Span], size: f32, flags: TextFlags) {
        for node in span {
            match node {
                Span::Text(text) => out.push(BodyNode::Text {
                    text: text.to_owned(),
                    flags,
                    size,
                }),
                Span::Code(text) => out.push(BodyNode::Text {
                    text: text.to_owned(),
                    flags: flags | TextFlags::MONOSPACE,
                    size,
                }),
                Span::Link(text, url, _) => out.push(BodyNode::Link {
                    text: text.to_owned(),
                    url: url.to_owned(),
                }),
                Span::Emphasis(vec) => span_text_inner(out, vec, size, flags | TextFlags::ITALIC),
                Span::Strong(vec) => span_text_inner(out, vec, size, flags | TextFlags::BOLD),
                _ => {}
            }
        }
    }

    let mut out = Vec::new();
    span_text_inner(&mut out, &span, size, flags);

    out
}
