// Experiment 11: author social/legacy ICO assets from canonical projections.
// Run root sync-nutorch-com-brand-assets.nu first. Normal builds use committed bytes.
import pngToIco from "png-to-ico";
import sharp from "sharp";
import { mkdir, writeFile } from "node:fs/promises";

const OUT = new URL("../public/images/", import.meta.url).pathname;
const PUB = new URL("../public/", import.meta.url).pathname;
const MARK = new URL(
  "../public/images/brand/nutorch-dark-400.webp",
  import.meta.url,
).pathname;

await mkdir(OUT, { recursive: true });

// The accepted colorful hero PNGs are intentionally not regenerated here.

// Favicon: 32px ICO.
const png32 = await sharp(`${PUB}/favicon-dark.png`).png().toBuffer();
await writeFile(`${PUB}/favicon.ico`, await pngToIco([png32]));

// OG image: 1200x630, dark brand background, mark + wordmark + tagline.
const og = sharp({
  create: {
    width: 1200,
    height: 630,
    channels: 4,
    background: { r: 17, g: 18, b: 25, alpha: 1 },
  },
});
const markLarge = await sharp(MARK).resize(340, 340).png().toBuffer();
const ogText = Buffer.from(`<svg width="1200" height="630">
  <text x="450" y="300" font-family="Helvetica, Arial, sans-serif"
    font-weight="bold" font-size="110">
    <tspan fill="#c0caf5">NuTorch</tspan>
  </text>
  <text x="452" y="370" font-family="Helvetica, Arial, sans-serif"
    font-size="40" fill="#9aa5ce">GPU tensors for every shell</text>
</svg>`);
await og
  .composite([
    { input: markLarge, left: 70, top: 145 },
    { input: ogText, left: 0, top: 0 },
  ])
  .png()
  .toFile(`${OUT}/og-nutorch.png`);

console.log("images processed → public/");
