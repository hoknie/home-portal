import { describe, expect, it } from "vitest";

import { terminalText } from "./terminal-text";

describe("terminalText", () => {
  it("keeps plain text as written", () => {
    expect(terminalText("one\ntwo\n")).toBe("one\ntwo\n");
  });

  it("treats a carriage return before a line feed as a line break", () => {
    expect(terminalText("one\r\ntwo\r\n")).toBe("one\ntwo\n");
  });

  it("a lone carriage return overwrites the start of the line like a terminal", () => {
    expect(terminalText("abcdef\rxy")).toBe("xycdef");
  });

  it("a trailing carriage return keeps the line", () => {
    expect(terminalText("done\r")).toBe("done");
  });

  it("curl's progress meter ends in its last state under its header", () => {
    const meter =
      "  % Total    % Received\n                                 Dload\n\r  0     0    0     0\r 45  4.5M   45  2.0M\r100  4.5M  100  4.5M\n";
    expect(terminalText(meter)).toBe("  % Total    % Received\n                                 Dload\n100  4.5M  100  4.5M\n");
  });

  it("removes colour and other escape sequences", () => {
    expect(terminalText("\u001b[32mok\u001b[0m and \u001b[1;31mbad\u001b[m")).toBe("ok and bad");
    expect(terminalText("\u001b]0;title\u0007text")).toBe("text");
    expect(terminalText("\u001b[2Kcleared")).toBe("cleared");
  });
});
