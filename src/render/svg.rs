use super::text::{render_text};
use crate::model::{Color, Node, Step};
use crate::render::core::RenderConfig;
use crate::render::layout::{ComputedLayout, LayoutContext};

use resvg::tiny_skia;
use std::rc::Rc;


use taffy::{prelude as tf, Taffy};

use usvg::{
    Fill,
};


pub(crate) struct RenderContext {
    step: Step,
    z_level: i32,
    layout: ComputedLayout,
    svg_node: usvg::Node,
}

impl From<&Color> for usvg::Color {
    fn from(value: &Color) -> Self {
        let c: svgtypes::Color = value.into();
        usvg::Color::new_rgb(c.red, c.green, c.blue)
    }
}

impl<'a> RenderContext {
    pub fn new(step: Step, z_level: i32, layout: ComputedLayout) -> Self {
        RenderContext {
            step,
            z_level,
            layout,
            svg_node: usvg::Node::new(usvg::NodeKind::Group(usvg::Group::default())),
        }
    }

    fn render_helper(&self, node: &Node) {
        if !node.show.at_step(self.step) {
            return;
        }
        if let Some(color) = &node.bg_color.at_step(self.step) {
            let (x, y, width, height) = self.layout.xywh(node.node_id);
            let mut path = usvg::Path::new(Rc::new(tiny_skia::PathBuilder::from_rect(
                tiny_skia::Rect::from_xywh(x, y, width, height).unwrap(),
            )));
            path.fill = Some(Fill {
                paint: usvg::Paint::Color(color.into()),
                ..Default::default()
            });
            self.svg_node
                .append(usvg::Node::new(usvg::NodeKind::Path(path)));
        }

        if let Some(text) = &node.text.at_step(self.step) {
            let (x, y) = self.layout.xy(node.node_id);
            self.svg_node.append(render_text(&text, x, y));
        }

        if let Some(children) = &node.children {
            for child in children {
                self.render_helper(child);
            }
        }
    }

    pub(crate) fn render_to_svg(self, node: &Node) -> usvg::Node {
        self.render_helper(node);
        self.svg_node
    }
}

pub(crate) fn render_to_svg_tree(render_cfg: &RenderConfig) -> usvg_tree::Tree {
    log::debug!("Creating layout");
    let layout_builder = LayoutContext::new(render_cfg.global_res, render_cfg.step);
    let layout = layout_builder.compute_layout(render_cfg.slide);

    println!("LAYOUT {:?}", layout);

    log::debug!("Rendering to svg");
    let render_ctx = RenderContext::new(render_cfg.step, 0, layout);
    let root_svg_node = render_ctx.render_to_svg(&render_cfg.slide.node);

    let size = usvg::Size::from_wh(render_cfg.slide.width, render_cfg.slide.height).unwrap();

    usvg_tree::Tree {
        size,
        view_box: usvg::ViewBox {
            rect: size.to_non_zero_rect(0.0, 0.0),
            aspect: usvg::AspectRatio::default(),
        },
        root: root_svg_node,
    }
}
