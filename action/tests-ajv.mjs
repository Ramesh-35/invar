// JUDGE-TESTS r1: the Ajv side of the two-validator check. Reads one JSON
// object on stdin, {base, head, instances: [{file, doc}]}, and prints one
// JSON object, {file: {base: A, head: A}}, where A is "valid", or
// {invalid: {pointer, message}}, or {error: reason}. Classification lives
// in tests.py; this file only answers. Strict mode off; formats not
// asserted, as in the judge.
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);

function compile(schema) {
  const draft4 = String(schema.$schema || "").includes("draft-04");
  const Ajv = draft4 ? require("ajv-draft-04") : require("ajv");
  const ajv = new Ajv({ strict: false, allErrors: false, validateFormats: false });
  require("ajv-formats")(ajv);
  return ajv.compile(schema);
}

function answer(validate, doc) {
  if (validate(doc)) return "valid";
  const e = validate.errors[0];
  return { invalid: { pointer: e.instancePath, message: e.message || "" } };
}

let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (c) => (input += c));
process.stdin.on("end", () => {
  const { base, head, instances } = JSON.parse(input);
  const sides = {};
  for (const [name, schema] of [["base", base], ["head", head]]) {
    try {
      sides[name] = compile(schema);
    } catch (e) {
      sides[name] = { error: `${e.name}: ${String(e.message).slice(0, 160)}` };
    }
  }
  const out = {};
  for (const { file, doc } of instances) {
    out[file] = {};
    for (const name of ["base", "head"]) {
      const v = sides[name];
      if (typeof v !== "function") { out[file][name] = v; continue; }
      try { out[file][name] = answer(v, doc); }
      catch (e) { out[file][name] = { error: `${e.name}: ${String(e.message).slice(0, 160)}` }; }
    }
  }
  process.stdout.write(JSON.stringify(out));
});
