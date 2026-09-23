"""Print the workspace crates that depend on the given one, transitively, plus itself."""

import json
import subprocess
import sys

target = sys.argv[1]
meta = json.loads(
    subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
)
members = {p["name"]: {d["name"] for d in p["dependencies"]} for p in meta["packages"]}

group = {target}
while True:
    grown = {name for name, deps in members.items() if deps & group}
    if grown <= group:
        break
    group |= grown

for name in sorted(group):
    print(name)
