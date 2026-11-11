// JUDGE-TESTS r1/r2: the Ajv side of the two-validator check. Reads one JSON
// object on stdin, {base, head, instances: [{file, doc}]}, and prints one
// JSON object, {file: {base: A, head: A}}, where A is "valid", or
// {invalid: {pointer, message}}, or {error: reason}. Classification lives
// in tests.py; this file only answers. Strict mode off; formats not
// asserted, as in the judge.
import { createRequire } from "node:module";
import { readFileSync, statSync } from "node:fs";
import { basename, join } from "node:path";
const require = createRequire(import.meta.url);

// JUDGE-TESTS r3: a $ref to another file is read from that side's siblings
// directory by the last segment of the URI, then with ".json" added. Only
// that directory is read; nothing is fetched.
function siblingPath(siblings, uri) {
  const target = uri.split("#")[0].split("?")[0].replace(/\/+$/, "");
  const name = basename(target.replace(/\\/g, "/"));
  if (!name || name === "." || name === "..") return null;
  const candidates = name.endsWith(".json") ? [name] : [name, name + ".json"];
  for (const c of candidates) {
    const p = join(siblings, c);
    try { if (statSync(p).isFile()) return p; } catch (_) { /* not there */ }
  }
  return null;
}

function loader(siblings) {
  return async (uri) => {
    const p = siblingPath(siblings, uri);
    if (p === null) throw new Error(`no file for ${uri} in the schema's directory`);
    return JSON.parse(readFileSync(p, "utf8"));
  };
}

// JUDGE-TESTS r2: the class follows the schema's $schema, as the Python
// side does. draft-04 -> ajv-draft-04; draft-06 -> Ajv with the draft-06
// meta-schema; 2019-09 -> Ajv2019; 2020-12 -> Ajv2020; draft-07, no
// $schema, or a URI naming none of these -> Ajv (draft-07), with the
// unknown URI dropped so Ajv does not try to resolve it.
function draftOf(uri) {
  const u = String(uri || "");
  if (u.includes("draft-04")) return "04";
  if (u.includes("draft-06")) return "06";
  if (u.includes("draft-07")) return "07";
  if (u.includes("2019-09")) return "2019";
  if (u.includes("2020-12")) return "2020";
  return u ? "unknown" : "07";
}

async function compile(schema, siblings) {
  const draft = draftOf(schema.$schema);
  const opts = { strict: false, allErrors: false, validateFormats: false, loadSchema: loader(siblings) };
  let ajv;
  if (draft === "04") ajv = new (require("ajv-draft-04"))(opts);
  else if (draft === "2019") ajv = new (require("ajv/dist/2019"))(opts);
  else if (draft === "2020") ajv = new (require("ajv/dist/2020"))(opts);
  else {
    ajv = new (require("ajv"))(opts);
    if (draft === "06") ajv.addMetaSchema(require("ajv/dist/refs/json-schema-draft-06.json"));
    if (draft === "unknown") {
      schema = { ...schema };
      delete schema.$schema;
    }
  }
  require("ajv-formats")(ajv);
  return ajv.compileAsync(schema);
}

function answer(validate, doc) {
  if (validate(doc)) return "valid";
  const e = validate.errors[0];
  return { invalid: { pointer: e.instancePath, message: e.message || "" } };
}

let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (c) => (input += c));
process.stdin.on("end", async () => {
  const { base, head, instances, base_siblings, head_siblings } = JSON.parse(input);
  const sides = {};
  for (const [name, schema, siblings] of [["base", base, base_siblings], ["head", head, head_siblings]]) {
    try {
      sides[name] = await compile(schema, siblings || ".");
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
