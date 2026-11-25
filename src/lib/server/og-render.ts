import satori, { type SatoriOptions } from "@cf-wasm/satori";
import { Resvg } from "@cf-wasm/resvg";
import { html } from "@xevion/satori-html";

export const OG_WIDTH = 1200;
export const OG_HEIGHT = 630;

/** Lays out markup with satori and rasterizes the SVG to a PNG at card size. */
export async function renderPng(
  markup: string,
  fonts: SatoriOptions["fonts"],
): Promise<Uint8Array> {
  const svg = await satori(html(markup), {
    width: OG_WIDTH,
    height: OG_HEIGHT,
    fonts,
  });
  const resvg = new Resvg(svg, { fitTo: { mode: "width", value: OG_WIDTH } });
  try {
    return resvg.render().asPng();
  } finally {
    resvg.free();
  }
}
