import { writeAll } from "./schemas/files.mjs";

const names = writeAll();
console.log(`generated ${names.length} response schemas`);
