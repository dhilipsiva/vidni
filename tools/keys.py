#!/usr/bin/env python3
"""Regenerate docs/keys.md, the reference of every key in vidni.toml, from
the project file's JSON Schema (docs/vidni.schema.json): `just keys`.

The schema is written by `just schema` from vidni's own project model, so
this page can't name a key that vidni doesn't read, or miss one it does.

usage: keys.py write|check
"""
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCHEMA = os.path.join(ROOT, "docs", "vidni.schema.json")
OUT = os.path.join(ROOT, "docs", "keys.md")

# The top-level tables, in the order a project file has them, and how each
# is written in TOML.
TOP = [
    ("meta", "[meta]"),
    ("voice", "[voice]"),
    ("timing", "[timing]"),
    ("lint", "[lint]"),
    ("style", "[style]"),
    ("cuts", "[[cuts]]"),
    ("music", "[music]"),
    ("publish", "[publish]"),
    ("sections", "[sections.<id>]"),
    ("lines", "[[lines]]"),
    ("scenes", "[scenes.<id>]"),
    ("expectations", "[[expectations]]"),
]


def one_line(text):
    return " ".join((text or "").split()).replace("|", "\\|")


class Reference:
    def __init__(self, schema):
        self.schema = schema
        self.defs = schema["$defs"]
        self.named = []  # the types that get a section of their own, in order of first use

    def resolve(self, node):
        """The definition `node` refers to, and its name; or the node itself."""
        ref = node.get("$ref")
        if ref:
            name = ref.rsplit("/", 1)[1]
            return self.defs[name], name
        return node, None

    def kind(self, node):
        """How a value is written, in a few words."""
        for key in ("anyOf", "oneOf"):
            if key in node:
                kinds = [self.kind(option) for option in node[key] if option.get("type") != "null"]
                return " or ".join(dict.fromkeys(kinds))
        target, name = self.resolve(node)
        if name:
            if "enum" in target:
                return " or ".join(f"`{json.dumps(v)}`" for v in target["enum"])
            if "oneOf" in target and all("const" in o or "enum" in o for o in target["oneOf"]):
                values = []
                for option in target["oneOf"]:
                    values += [option["const"]] if "const" in option else option["enum"]
                return " or ".join(f"`{json.dumps(v)}`" for v in values)
            if target.get("type") == "object" and "properties" in target:
                if name not in self.named:
                    self.named.append(name)
                return f"[{name}](#{name.lower()})"
            return self.kind(target)
        types = node.get("type")
        if isinstance(types, list):
            types = [t for t in types if t != "null"]
            types = types[0] if len(types) == 1 else types
        if types == "array":
            items = node.get("items")
            if isinstance(items, dict):
                return f"array of {self.kind(items)}"
            if "prefixItems" in node:
                return "[" + ", ".join(self.kind(i) for i in node["prefixItems"]) + "]"
            return "array"
        if types == "object":
            more = node.get("additionalProperties")
            if isinstance(more, dict):
                return f"table of {self.kind(more)}, by name"
            return "table"
        if types == "number":
            return "number"
        if types == "integer":
            return "whole number"
        if isinstance(types, list):
            return " or ".join(types)
        return types or "any"

    def table(self, target):
        required = set(target.get("required", []))
        rows = []
        for key, node in target.get("properties", {}).items():
            need = "yes" if key in required else ""
            text = one_line(node.get("description") or self.resolve(node)[0].get("description"))
            rows.append(f"| `{key}` | {self.kind(node)} | {need} | {text} |")
        # Required keys first, then the rest, each in the schema's order.
        rows.sort(key=lambda row: "| yes |" not in row)
        return "\n".join(["| Key | Value | Required | Meaning |", "|---|---|---|---|"] + rows)

    def page(self):
        parts = ["""# Every key of `vidni.toml`

This page lists each key vidni reads from a project file, with its type and its meaning. It is generated from the project file's JSON Schema ([vidni.schema.json](vidni.schema.json)), which vidni writes from its own model of a project: `just keys` writes this page again, and `just keys check` fails when it is out of date. Don't edit it by hand.

[The spec](spec.md) explains how the keys work together, with examples. A key that isn't on this page is an error: `vidni validate` names it, with its line and column.

A project file starts with `schema = 1`, the version of this layout."""]
        top = self.schema["properties"]
        for key, written in TOP:
            node = top[key]
            target = node
            for inner in ("items", "additionalProperties"):
                if isinstance(target.get(inner), dict):
                    target = target[inner]
            target, _ = self.resolve(target)
            parts.append(f"## `{written}`\n\n{one_line(node.get('description'))}\n\n{self.table(target)}")
        missing = set(top) - {key for key, _ in TOP} - {"schema"}
        if missing:
            sys.exit(f"tools/keys.py doesn't place these top-level keys: {sorted(missing)}")
        parts.append("## Values with keys of their own\n\nThese are written inline, as `{ key = value }`, or as a table of their own.")
        done = 0
        while done < len(self.named):
            name = self.named[done]
            done += 1
            target = self.defs[name]
            parts.append(f"### {name}\n\n{one_line(target.get('description'))}\n\n{self.table(target)}")
        return "\n\n".join(parts) + "\n"


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in ("write", "check"):
        sys.exit(__doc__)
    with open(SCHEMA) as f:
        text = Reference(json.load(f)).page()
    if sys.argv[1] == "write":
        with open(OUT, "w") as f:
            f.write(text)
        print("docs/keys.md: written")
        return
    current = open(OUT).read() if os.path.exists(OUT) else ""
    if current != text:
        sys.exit("docs/keys.md is out of date; run `just keys`")
    print("docs/keys.md: up to date")


if __name__ == "__main__":
    main()
