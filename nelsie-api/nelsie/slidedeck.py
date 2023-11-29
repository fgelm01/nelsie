import os
from importlib import resources
from typing import Optional

from .box import Box, BoxBuilder, BoxChild
from .export import ExportSlide, ExportSlideDeck
from .render import render_slides
from .serialization import serialize
from .text.manager import TextStyleManager, TextStylesProviderMixin
from .text.textstyle import DEFAULT_STYLE


class Slide(BoxBuilder, TextStylesProviderMixin):
    def __init__(
        self,
        style_manager: TextStyleManager,
        width: float,
        height: float,
        bg_color: str,
        image_directory: str | None,
        debug_layout: bool | str,
        name: str,
    ):
        self.style_manager = style_manager
        self.width = width
        self.height = height
        self.n_steps = 0
        self.box_id_counter = 0
        self.image_directory = image_directory
        self.debug_layout = debug_layout
        self.root_box = Box(
            slide=self,
            parent_id=None,
            # As this box is hidden and exposed only through Slide, do NOT copy, but share it directly
            style_manager=self.style_manager,
            show=True,
            x=None,
            y=None,
            p_left=0,
            p_right=0,
            p_top=0,
            p_bottom=0,
            m_left=0,
            m_right=0,
            m_top=0,
            m_bottom=0,
            z_level=0,
            width=self.width,
            height=self.height,
            row=False,
            reverse=False,
            bg_color=bg_color,
            name=name,
            debug_layout=debug_layout,
        )

    def new_box_id(self):
        self.box_id_counter += 1
        return self.box_id_counter

    def get_slide(self):
        return self

    def get_box(self):
        return self.root_box

    def add_child(self, child: BoxChild):
        self.root_box.add_child(child)

    def update_min_steps(self, n_steps: int):
        self.n_steps = max(self.n_steps, n_steps)

    def export(self) -> ExportSlide:
        return ExportSlide(
            width=self.width,
            height=self.height,
            node=self.root_box.export(),
            n_steps=self.n_steps,
        )


class SlideDeck(TextStylesProviderMixin):
    def __init__(
        self,
        *,
        builder_bin_path: str | None = None,
        width: float = 1024,
        height: float = 768,
        bg_color: str = "white",
        image_directory: str | None = None
    ):
        if builder_bin_path is None:
            from . import backend
            with resources.path(backend, ".") as path:
                builder_bin_path = os.path.join(path, "nelsie-builder")

        self.builder_bin_path = builder_bin_path
        self.width = width
        self.height = height
        self.bg_color = bg_color
        self.image_directory = image_directory

        self.style_manager = TextStyleManager({"default": DEFAULT_STYLE})
        self.slides: list[Slide] = []

    def new_slide(
        self,
        width: Optional[float] = None,
        height: Optional[float] = None,
        bg_color: Optional[str] = None,
        image_directory: str | None = None,
        name: str = "",
        debug_layout: bool = False,
    ):
        slide = Slide(
            style_manager=self.style_manager.copy(),
            width=width or self.width,
            height=height or self.height,
            bg_color=bg_color or self.bg_color,
            image_directory=image_directory or self.image_directory,
            name=name,
            debug_layout=debug_layout,
        )
        self.slides.append(slide)
        return slide

    def slide(
        self,
        width: Optional[float] = None,
        height: Optional[float] = None,
        bg_color: Optional[str] = None,
        image_directory: str | None = None,
        name: str = "",
        debug_layout: bool = False,
    ):
        def helper(fn):
            slide = self.new_slide(
                width, height, bg_color, image_directory, name, debug_layout
            )
            return fn(slide)

        return helper

    def export(self) -> ExportSlideDeck:
        return ExportSlideDeck(slides=[slide.export() for slide in self.slides])

    def render(
        self,
        *,
        output_pdf: Optional[str] = None,
        output_svg: Optional[str] = None,
        output_png: Optional[str] = None,
        debug: bool = False
    ):
        if output_pdf is None and output_png is None and output_svg is None:
            raise Exception("No output file is defined")
        render_slides(
            self.builder_bin_path,
            serialize(self.export()),
            output_pdf,
            output_svg,
            output_png,
            debug,
        )
