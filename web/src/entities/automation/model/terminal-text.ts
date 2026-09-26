const ESCAPE_SEQUENCES = new RegExp(
  [
    String.raw`\u001b\][^\u0007\u001b]*(?:\u0007|\u001b\\)`,
    String.raw`\u001b\[[0-?]*[ -/]*[@-~]`,
    String.raw`\u001b[@-Z\\-_]`,
  ].join("|"),
  "g",
);

function overwrite(line: string) {
  const segments = line.split("\r");
  let screen = "";
  for (const segment of segments) {
    screen = segment + screen.slice(segment.length);
  }
  return screen;
}

export function terminalText(raw: string) {
  return raw.replace(ESCAPE_SEQUENCES, "").replace(/\r\n/g, "\n").split("\n").map(overwrite).join("\n");
}
