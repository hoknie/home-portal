export function quoted(argument: string) {
  return `'${argument.replace(/'/g, `'\\''`)}'`;
}

export function quotedCommand(script: string, args: string[]) {
  return [script, ...args.map(quoted)].join(" ");
}
