use crate::common::Step;
use crate::model::{Node, NodeChild, NodeId, PartialTextStyle, Slide, SlideDeck};
use crate::parsers::parse_color;
use crate::pyinterface::insteps::ValueOrInSteps;
use crate::pyinterface::r#box::{BoxConfig, NodeCreationEnv};
use crate::pyinterface::resources::Resources;
use crate::pyinterface::textstyle::PyTextStyle;
use crate::render::{render_slide_deck, OutputConfig};
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::{pyclass, pymethods, PyObject, PyResult, Python, ToPyObject};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

#[pyclass]
pub(crate) struct Deck {
    deck: SlideDeck,
}

type SlideId = u32;
type BoxId = Vec<u32>;

fn resolve_box_id<'a>(node: &'a mut Node, box_id: &[u32]) -> Option<&'a mut Node> {
    if box_id.is_empty() {
        return Some(node);
    }
    node.child_node_mut(box_id[0] as usize)
        .and_then(|child| resolve_box_id(child, &box_id[1..]))
}

// fn resolve_box_id_to_node_ids<'a>(
//     node: &'a mut Node,
//     box_id: &[u32],
// ) -> Option<(&'a mut Node, Vec<NodeId>)> {
//     let mut predecessors_ids = Vec::with_capacity(box_id.len() + 1);
//     predecessors_ids.push(node.node_id);
//     let mut node = node;
//     for id in box_id {
//         if let Some(child) = node.child_node_mut(*id as usize) {
//             predecessors_ids.push(child.node_id);
//             node = child;
//         } else {
//             return None;
//         }
//     }
//     return Some((node, predecessors_ids));
// }

#[pymethods]
impl Deck {
    #[new]
    fn new(resources: &Resources, default_font: Option<&str>) -> PyResult<Self> {
        Ok(Deck {
            deck: SlideDeck::new(&resources.resources, default_font)?,
        })
    }

    fn new_slide(
        &mut self,
        width: f32,
        height: f32,
        bg_color: &str,
        name: String,
    ) -> PyResult<SlideId> {
        let slide_id = self.deck.slides.len() as SlideId;
        self.deck.slides.push(Slide::new(
            width,
            height,
            name,
            parse_color(bg_color)?,
            self.deck.global_styles.clone(),
        ));
        Ok(slide_id)
    }

    fn new_box(
        &mut self,
        resources: &mut Resources,
        slide_id: SlideId,
        box_id: BoxId,
        config: BoxConfig,
    ) -> PyResult<(BoxId, u32)> {
        let slide = self
            .deck
            .slides
            .get_mut(slide_id as usize)
            .ok_or_else(|| PyException::new_err("Invalid slide id"))?;
        let node_id = slide.new_node_id();
        let parent_node = resolve_box_id(&mut slide.node, &box_id)
            .ok_or_else(|| PyException::new_err("Invalid box id"))?;

        let mut nce = NodeCreationEnv {
            resources: &mut resources.resources,
        };
        let (node, n_steps) = config.make_node(
            node_id,
            parent_node.node_id,
            &mut nce,
            parent_node.styles.clone(),
        )?;
        slide.n_steps = slide.n_steps.max(n_steps);

        let new_id = parent_node.children.len() as u32;
        let node_id = node.node_id;
        parent_node.children.push(NodeChild::Node(node));

        let mut new_box_id = box_id;
        new_box_id.push(new_id);
        Ok((new_box_id, node_id.as_u32()))
    }

    fn set_style(
        &mut self,
        resources: &Resources,
        name: &str,
        text_style: ValueOrInSteps<PyTextStyle>,
        slide_id: Option<SlideId>,
        box_id: Option<BoxId>,
    ) -> PyResult<()> {
        if let Some(slide_id) = slide_id {
            let slide = self
                .deck
                .slides
                .get_mut(slide_id as usize)
                .ok_or_else(|| PyException::new_err("Invalid slide id"))?;
            if let Some(box_id) = box_id {
                let node = resolve_box_id(&mut slide.node, &box_id)
                    .ok_or_else(|| PyException::new_err("Invalid node id"))?;
                let text_style = text_style.parse(&mut slide.n_steps, |s| {
                    s.to_partial_style(&resources.resources)
                })?;
                Arc::make_mut(&mut node.styles).set_style(name.to_string(), text_style);
            }
        } else {
            let text_style =
                text_style.parse_ignore_n_steps(|s| s.to_partial_style(&resources.resources))?;
            Arc::make_mut(&mut self.deck.global_styles).set_style(name.to_string(), text_style);
        }
        Ok(())
    }

    fn get_style(
        &mut self,
        py: Python<'_>,
        name: &str,
        step: Step,
        slide_id: Option<SlideId>,
        box_id: Option<BoxId>,
    ) -> PyResult<PyObject> {
        Ok((if let Some(slide_id) = slide_id {
            let slide = self
                .deck
                .slides
                .get_mut(slide_id as usize)
                .ok_or_else(|| PyException::new_err("Invalid slide id"))?;
            if let Some(box_id) = box_id {
                let node = resolve_box_id(&mut slide.node, &box_id)
                    .ok_or_else(|| PyException::new_err("Invalid node id"))?;
                node.styles
                    .get_style(name)
                    .map(|style| PyTextStyle::new(style.at_step(step).clone()))?
            } else {
                return Err(PyException::new_err("Invalid box id"));
            }
        } else {
            self.deck
                .global_styles
                .get_style(name)
                .map(|style| PyTextStyle::new(style.at_step(step).clone()))?
        })
        .to_object(py))
    }

    fn render(
        &self,
        resources: &mut Resources,
        output_pdf: Option<&str>,
        output_svg: Option<&str>,
        output_png: Option<&str>,
    ) -> PyResult<()> {
        Ok(render_slide_deck(
            &self.deck,
            &resources.resources,
            &OutputConfig {
                output_pdf: output_pdf.map(|p| std::path::Path::new(p)),
                output_png: output_png.map(|p| std::path::Path::new(p)),
                output_svg: output_svg.map(|p| std::path::Path::new(p)),
            },
        )?)
    }
}