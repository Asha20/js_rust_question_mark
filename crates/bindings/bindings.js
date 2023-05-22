import { createRequire } from "module";

const mod = createRequire(import.meta.url)("./dist/index.node");
export const process = mod.process;
