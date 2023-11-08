import dataclasses
from typing import Optional

from .export import ExportSlideDeck, ExportSlide
from .box import Box, BoxBuilder
from .render import render_slides


class Slide(BoxBuilder):
    def __init__(self, width: float, height: float, bg_color: str):
        self.width = width
        self.height = height
        self.n_steps = 0
        self.root_box = Box(
            self,
            show=True,
            width=self.width,
            height=self.height,
            row=False,
            reverse=False,
            bg_color=bg_color,
        )

    def get_slide(self):
        return self

    def add_box(self, box: Box):
        self.root_box.add_box(box)

    def update_min_steps(self, n_steps: int):
        self.n_steps = max(self.n_steps, n_steps)

    def export(self) -> ExportSlide:
        return ExportSlide(
            width=self.width,
            height=self.height,
            node=self.root_box.export(),
            n_steps=self.n_steps,
        )


class SlideDeck:
    def __init__(
        self,
        *,
        nelsie_bin: str,
        width: float = 1024,
        height: float = 768,
        bg_color: str = "white"
    ):
        self.nelsie_bin = nelsie_bin
        self.width = width
        self.height = height
        self.bg_color = bg_color

        self.slides: list[Slide] = []

    def new_slide(
        self,
        width: Optional[float] = None,
        height: Optional[float] = None,
        bg_color: Optional[str] = None,
    ):
        slide = Slide(
            width=width or self.width,
            height=height or self.height,
            bg_color=bg_color or self.bg_color,
        )
        self.slides.append(slide)
        return slide

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
            self.nelsie_bin,
            dataclasses.asdict(self.export()),
            output_pdf,
            output_svg,
            output_png,
            debug,
        )
