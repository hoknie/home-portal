export type Color = { red: number; green: number; blue: number; alpha: number };

const OKLCH = /^oklch\(\s*([\d.]+)(%?)\s+([\d.]+)\s+([\d.]+)\s*(?:\/\s*([\d.]+)(%?))?\s*\)$/;

function clamp(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function encode(linear: number): number {
  const value = clamp(linear);
  return value <= 0.0031308 ? 12.92 * value : 1.055 * value ** (1 / 2.4) - 0.055;
}

function decode(encoded: number): number {
  return encoded <= 0.04045 ? encoded / 12.92 : ((encoded + 0.055) / 1.055) ** 2.4;
}

export function parseOklch(text: string): Color {
  const match = OKLCH.exec(text.trim());
  if (!match) {
    throw new Error(`not an oklch colour: ${text}`);
  }
  const lightness = Number(match[1]) / (match[2] ? 100 : 1);
  const chroma = Number(match[3]);
  const hue = (Number(match[4]) * Math.PI) / 180;
  const alpha = match[5] === undefined ? 1 : Number(match[5]) / (match[6] ? 100 : 1);
  const a = chroma * Math.cos(hue);
  const b = chroma * Math.sin(hue);
  const long = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const medium = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const short = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return {
    red: encode(4.0767416621 * long - 3.3077115913 * medium + 0.2309699292 * short),
    green: encode(-1.2684380046 * long + 2.6097574011 * medium - 0.3413193965 * short),
    blue: encode(-0.0041960863 * long - 0.7034186147 * medium + 1.707614701 * short),
    alpha,
  };
}

export function composite(top: Color, bottom: Color): Color {
  const mix = (upper: number, lower: number) => upper * top.alpha + lower * (1 - top.alpha);
  return {
    red: mix(top.red, bottom.red),
    green: mix(top.green, bottom.green),
    blue: mix(top.blue, bottom.blue),
    alpha: top.alpha + bottom.alpha * (1 - top.alpha),
  };
}

export function relativeLuminance(color: Color): number {
  return 0.2126 * decode(color.red) + 0.7152 * decode(color.green) + 0.0722 * decode(color.blue);
}

export function contrastRatio(first: Color, second: Color): number {
  const lighter = Math.max(relativeLuminance(first), relativeLuminance(second));
  const darker = Math.min(relativeLuminance(first), relativeLuminance(second));
  return (lighter + 0.05) / (darker + 0.05);
}
