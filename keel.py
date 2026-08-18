#!/usr/bin/env python3
"""keel — the tool behind the Keel method.

One file, standard library only. It knows state; it writes no prose.
Messages printed to the human stay in Ukrainian: they are prose, not code.

    keel new step <slug>       skeleton of a step
    keel new contract <slug>   skeleton of a contract
    keel plan                  completeness of a plan
    keel next                  package for the next move
    keel check                 the six checks
    keel rev                   revisions that have drifted apart
    keel hooks                 git hooks: pre-commit and pre-push
    keel init                  put Keel into a project
"""

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

VERSION = "0.1.0"

REV_LEN = 6          # how many hex digits keel rev writes
REV_MIN = 4          # a shorter revision in a reference is not accepted


# ─────────────────────────────────────────────────────────────────────────────
# YAML: a narrow subset
#
# Indented block maps, block lists, flow [a, b] and {k: v}, quotes, comments.
# No anchors, no multi-line scalars, no types. Exactly as much as a Keel
# document header needs — and not one pip install.
# ─────────────────────────────────────────────────────────────────────────────

class YamlError(Exception):
    def __init__(self, line, message):
        super().__init__(f"рядок {line}: {message}")
        self.line = line
        self.message = message


def _strip_comment(text):
    out = []
    quote = None
    for ch in text:
        if quote:
            out.append(ch)
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
            out.append(ch)
        elif ch == "#" and (not out or out[-1] in " \t"):
            break
        else:
            out.append(ch)
    return "".join(out).rstrip()


def _scalar(text, line):
    text = text.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        return text[1:-1]
    if text.startswith(("\"", "'")):
        raise YamlError(line, "лапки не закриті")
    return text


def _split_flow(text, line):
    """Split the inside of [..] or {..} on top-level commas."""
    parts, depth, quote, cur = [], 0, None, []
    for ch in text:
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
            if depth < 0:
                raise YamlError(line, "зайва дужка")
        elif ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
            continue
        cur.append(ch)
    if quote:
        raise YamlError(line, "лапки не закриті")
    if depth:
        raise YamlError(line, "дужка не закрита")
    tail = "".join(cur).strip()
    if tail:
        parts.append(tail)
    return [p.strip() for p in parts if p.strip()]


def _flow(text, line):
    text = text.strip()
    if text.startswith("["):
        if not text.endswith("]"):
            raise YamlError(line, "список не закритий дужкою")
        return [_flow(p, line) for p in _split_flow(text[1:-1], line)]
    if text.startswith("{"):
        if not text.endswith("}"):
            raise YamlError(line, "мапа не закрита дужкою")
        out = {}
        for part in _split_flow(text[1:-1], line):
            if ":" not in part:
                raise YamlError(line, f"у мапі немає двокрапки: {part!r}")
            key, _, value = part.partition(":")
            key = _scalar(key, line)
            if key in out:
                raise YamlError(line, f"ключ {key!r} повторюється")
            out[key] = _flow(value, line)
        return out
    return _scalar(text, line)


def parse_yaml(text):
    """Parse a document header. Returns a dict."""
    lines = []
    for number, raw in enumerate(text.splitlines(), 1):
        if "\t" in raw[: len(raw) - len(raw.lstrip())]:
            raise YamlError(number, "відступ табуляцією")
        body = _strip_comment(raw)
        if not body.strip():
            continue
        lines.append((number, len(body) - len(body.lstrip(" ")), body.strip()))

    value, index = _parse_block(lines, 0, 0)
    if index != len(lines):
        raise YamlError(lines[index][0], "несподіваний відступ")
    return value if isinstance(value, dict) else {}


def _parse_block(lines, index, indent):
    if index >= len(lines):
        return {}, index
    if lines[index][2].startswith("- "):
        return _parse_list(lines, index, indent)
    return _parse_map(lines, index, indent)


def _parse_list(lines, index, indent):
    items = []
    while index < len(lines):
        number, own, text = lines[index]
        if own < indent:
            break
        if own > indent or not text.startswith("- "):
            raise YamlError(number, "рядок не є елементом списку")
        items.append(_flow(text[2:], number))
        index += 1
    return items, index


def _parse_map(lines, index, indent):
    out = {}
    while index < len(lines):
        number, own, text = lines[index]
        if own < indent:
            break
        if own > indent:
            raise YamlError(number, "несподіваний відступ")
        if ":" not in text:
            raise YamlError(number, f"немає двокрапки: {text!r}")
        key, _, rest = text.partition(":")
        key = _scalar(key, number)
        if not key:
            raise YamlError(number, "порожній ключ")
        if key in out:
            raise YamlError(number, f"ключ {key!r} повторюється")
        index += 1
        rest = rest.strip()
        if rest:
            out[key] = _flow(rest, number)
            continue
        if index < len(lines) and lines[index][1] > indent:
            nested, index = _parse_block(lines, index, lines[index][1])
            out[key] = nested
        elif index < len(lines) and lines[index][2].startswith("- ") and lines[index][1] == indent:
            out[key], index = _parse_list(lines, index, indent)
        else:
            out[key] = None
    return out, index


# ─────────────────────────────────────────────────────────────────────────────
# Documents: header, body, sections, revisions
# ─────────────────────────────────────────────────────────────────────────────

SECTION_RE = re.compile(r"^##\s+(.+?)\s*$", re.M)
LINK_RE = re.compile(r"\]\(([^)\s]+\.md)\)")


def revision(text, length=REV_LEN):
    """Short hash of a text. Only repeated spaces and newlines are collapsed."""
    return full_revision(text)[:length]


def full_revision(text):
    return hashlib.sha256(re.sub(r"\s+", " ", text).strip().encode("utf-8")).hexdigest()


def rev_matches(recorded, text):
    if not recorded or len(recorded) < REV_MIN:
        return False
    return full_revision(text).startswith(recorded.lower())


class Ref:
    """A reference of the form `slug` or `slug@a3f1c0`."""

    __slots__ = ("slug", "rev", "raw")

    def __init__(self, raw):
        self.raw = raw = str(raw).strip()
        self.slug, _, self.rev = raw.partition("@")
        self.slug = self.slug.strip()
        self.rev = self.rev.strip() or None

    def __repr__(self):
        return f"Ref({self.raw!r})"


class Doc:
    def __init__(self, path, root):
        self.path = path
        self.rel = os.path.relpath(path, root).replace(os.sep, "/")
        self.slug = os.path.splitext(os.path.basename(path))[0]
        self.error = None
        self.front = {}
        self.body = ""
        self.sections = {}          # heading -> the text under it
        self.section_lines = {}     # heading -> line number
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        self.text = text
        front_text, self.body, self.body_offset = split_front_matter(text)
        if front_text is None:
            self.error = "немає шапки між рисками ---"
            return
        try:
            self.front = parse_yaml(front_text) or {}
        except YamlError as exc:
            self.error = f"шапка не читається: {exc}"
            return
        self._split_sections()

    def _split_sections(self):
        marks = list(SECTION_RE.finditer(self.body))
        for order, mark in enumerate(marks):
            end = marks[order + 1].start() if order + 1 < len(marks) else len(self.body)
            title = mark.group(1).strip()
            self.sections[title] = self.body[mark.end():end].strip()
            self.section_lines[title] = self.body[: mark.start()].count("\n") + self.body_offset + 1

    def named_sections(self, kind):
        """Sections of the form `## scenario: slug` -> {slug: text}."""
        out = {}
        for title, text in self.sections.items():
            head, _, slug = title.partition(":")
            if head.strip().lower() == kind and slug.strip():
                out[slug.strip()] = text
        return out

    def line_of(self, needle):
        for number, line in enumerate(self.text.splitlines(), 1):
            if needle in line:
                return number
        return 1


def split_front_matter(text):
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None, text, 1
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            front = "\n".join(lines[1:index])
            body = "\n".join(lines[index + 1:])
            return front, body, index + 2
    return None, text, 1


class Contract(Doc):
    @property
    def module(self):
        value = self.front.get("module")
        return value.strip() if isinstance(value, str) else None

    @property
    def exports(self):
        value = self.front.get("exports")
        if isinstance(value, list):
            return [str(item).strip() for item in value if str(item).strip()]
        return []

    @property
    def revision(self):
        """A contract's revision covers the whole file, header along with body."""
        return revision(self.text)

    def rev_ok(self, recorded):
        return rev_matches(recorded, self.text)


class Step(Doc):
    @property
    def depends_on(self):
        value = self.front.get("depends_on")
        return [Ref(item) for item in value] if isinstance(value, list) else []

    @property
    def scenarios(self):
        value = self.front.get("scenarios")
        return value if isinstance(value, dict) else {}

    @property
    def transforms(self):
        value = self.front.get("transforms")
        return value if isinstance(value, dict) else {}

    def scenario_body(self, slug):
        return self.named_sections("scenario").get(slug)

    def transform_body(self, slug):
        return self.named_sections("transform").get(slug)

    def scenario_revision(self, slug):
        body = self.scenario_body(slug)
        return revision(body) if body is not None else None

    def proves(self, slug):
        spec = self.scenarios.get(slug)
        if isinstance(spec, dict) and spec.get("proves"):
            value = spec["proves"]
            values = value if isinstance(value, list) else [value]
            return [Ref(item) for item in values]
        return []

    def transform_files(self, slug):
        spec = self.transforms.get(slug)
        if not isinstance(spec, dict):
            return []
        value = spec.get("files")
        if isinstance(value, list):
            return [str(item).strip() for item in value if str(item).strip()]
        if isinstance(value, str) and value.strip():
            return [value.strip()]
        return []

    def transform_contracts(self, slug):
        spec = self.transforms.get(slug)
        if not isinstance(spec, dict):
            return []
        value = spec.get("contracts")
        if isinstance(value, list):
            return [Ref(item) for item in value]
        if isinstance(value, str) and value.strip():
            return [Ref(value)]
        return []

    def transform_implements(self, slug):
        spec = self.transforms.get(slug)
        if not isinstance(spec, dict):
            return []
        value = spec.get("implements")
        if isinstance(value, list):
            return [str(item).strip() for item in value if str(item).strip()]
        if isinstance(value, str) and value.strip():
            return [value.strip()]
        return []

    @property
    def why(self):
        for title, text in self.sections.items():
            if title.strip().lower() in ("навіщо", "why"):
                return text
        return ""


# ─────────────────────────────────────────────────────────────────────────────
# git
# ─────────────────────────────────────────────────────────────────────────────

class Git:
    def __init__(self, root):
        self.root = root

    def run(self, *args):
        proc = subprocess.run(
            ["git", "-C", self.root, *args],
            capture_output=True, text=True,
        )
        return proc.returncode, proc.stdout, proc.stderr

    def out(self, *args, default=""):
        code, stdout, _ = self.run(*args)
        return stdout.strip() if code == 0 else default

    @property
    def available(self):
        return self.run("rev-parse", "--git-dir")[0] == 0

    @property
    def branch(self):
        return self.out("rev-parse", "--abbrev-ref", "HEAD")

    @property
    def main_branch(self):
        """The main branch. On CI it is not local — there it is origin/main."""
        head = self.out("symbolic-ref", "--quiet", "refs/remotes/origin/HEAD")
        if head:
            short = head.rsplit("/", 1)[-1]
            if self.run("rev-parse", "--verify", "--quiet", short)[0] == 0:
                return short
            return f"origin/{short}"
        for name in ("main", "master", "origin/main", "origin/master"):
            if self.run("rev-parse", "--verify", "--quiet", name)[0] == 0:
                return name
        return "main"

    @property
    def main_short(self):
        return self.main_branch.rsplit("/", 1)[-1]

    def merge_base(self, other):
        return self.out("merge-base", other, "HEAD")

    def changed_files(self, base):
        """All the branch changed: commits since the base plus what is not committed."""
        files = set()
        if base:
            code, stdout, _ = self.run("diff", "--name-only", base, "HEAD")
            if code == 0:
                files.update(name for name in stdout.splitlines() if name)
        code, stdout, _ = self.run("status", "--porcelain", "-z", "--untracked-files=all")
        if code == 0:
            fields = [item for item in stdout.split("\0") if item]
            index = 0
            while index < len(fields):
                entry = fields[index]
                status, name = entry[:2], entry[3:]
                index += 1
                if status[0] in "RC" and index < len(fields):
                    files.add(fields[index])
                    index += 1
                if name:
                    files.add(name)
        return files

    def commits_since(self, base):
        """[(sha, message, {files})], oldest first."""
        if not base:
            return []
        code, stdout, _ = self.run("log", "--format=%H%x1f%B%x1e", f"{base}..HEAD")
        if code != 0:
            return []
        commits = []
        for chunk in stdout.split("\x1e"):
            chunk = chunk.strip("\n")
            if not chunk.strip():
                continue
            sha, _, message = chunk.partition("\x1f")
            sha = sha.strip()
            files = set(self.out("show", "--name-only", "--format=", sha).splitlines())
            commits.append((sha, message.strip(), {name for name in files if name}))
        commits.reverse()
        return commits

    def file_in_branch(self, branch, path):
        return self.run("cat-file", "-e", f"{branch}:{path}")[0] == 0


# ─────────────────────────────────────────────────────────────────────────────
# Language adapters
#
# Two of the six checks depend on the language: what runs the tests and where
# a module's exports come from. The adapter is chosen by a marker in the root.
# ─────────────────────────────────────────────────────────────────────────────

EXPORT_MARK = "keel-exports|"


class Adapter:
    name = "?"
    marker = ()
    test_dirs = ()
    test_suffix = ()
    tag_re = None

    @classmethod
    def detect(cls, root):
        return any(os.path.exists(os.path.join(root, item)) for item in cls.marker)

    def test_command(self):
        raise NotImplementedError

    def test_files(self, root):
        found = []
        for directory in self.test_dirs:
            base = os.path.join(root, directory)
            for current, _, names in os.walk(base):
                for name in sorted(names):
                    if name.endswith(tuple(self.test_suffix)):
                        found.append(os.path.join(current, name))
        return sorted(found)

    def tags(self, root):
        """{scenario -> [(file, line, revision)]}, slug normalised."""
        out = {}
        for path in self.test_files(root):
            try:
                with open(path, encoding="utf-8") as handle:
                    text = handle.read()
            except OSError:
                continue
            for match in self.tag_re.finditer(text):
                slug = normalise_slug(match.group(1))
                line = text[: match.start()].count("\n") + 1
                out.setdefault(slug, []).append(
                    (os.path.relpath(path, root).replace(os.sep, "/"), line, match.group(2))
                )
        return out

    def exports(self, root, modules):
        raise NotImplementedError

    def ci_steps(self, root):
        """Workflow lines that install the language. Without them CI is mute."""
        return []


def normalise_slug(text):
    return re.sub(r"[^a-z0-9]+", "-", str(text).strip().lower()).strip("-")


class ElixirAdapter(Adapter):
    name = "elixir"
    marker = ("mix.exs",)
    test_dirs = ("test",)
    test_suffix = ("_test.exs",)
    # rev is captured whatever it looks like, not only hex: rubbish in a
    # revision should turn a check red rather than pass unnoticed.
    tag_re = re.compile(
        r"@tag\s+proves:\s*:([A-Za-z0-9_?!]+)"
        r"(?:\s*,\s*rev:\s*[\"']([^\"']*)[\"'])?"
    )

    def test_command(self):
        return ["mix", "test"]

    def ci_steps(self, root):
        elixir, otp = self.versions()
        return [
            "      - uses: erlef/setup-beam@v1",
            "        with:",
            f"          elixir-version: '{elixir}'",
            f"          otp-version: '{otp}'",
            "      - run: mix deps.get",
        ]

    @staticmethod
    def versions():
        """Versions from the machine running init. We ask rather than guess."""
        try:
            proc = subprocess.run(["elixir", "--version"],
                                  capture_output=True, text=True, timeout=60)
        except (OSError, subprocess.SubprocessError):
            return "1.18", "27"
        otp = re.search(r"Erlang/OTP (\d+)", proc.stdout)
        elixir = re.search(r"Elixir (\d+\.\d+)", proc.stdout)
        return (elixir.group(1) if elixir else "1.18",
                otp.group(1) if otp else "27")

    def exports(self, root, modules):
        if not modules:
            return {}
        listing = ", ".join(f'"{name}"' for name in modules)
        script = (
            f"for name <- [{listing}] do\n"
            "  mod = Module.concat([name])\n"
            "  if Code.ensure_loaded?(mod) do\n"
            "    funs = mod.__info__(:functions) ++ mod.__info__(:macros)\n"
            "    body = Enum.map_join(funs, \",\", fn {f, a} -> \"#{f}/#{a}\" end)\n"
            f"    IO.puts(\"{EXPORT_MARK}\" <> name <> \"|\" <> body)\n"
            "  else\n"
            f"    IO.puts(\"{EXPORT_MARK}\" <> name <> \"|__missing__\")\n"
            "  end\n"
            "end\n"
        )
        proc = subprocess.run(
            ["mix", "run", "--no-start", "-e", script],
            cwd=root, capture_output=True, text=True,
        )
        return parse_export_output(proc, modules)


class PythonAdapter(Adapter):
    name = "python"
    marker = ("pyproject.toml", "setup.py", "setup.cfg")
    test_dirs = ("tests", "test")
    test_suffix = ("_test.py",)
    tag_re = re.compile(
        r"#\s*proves:\s*([A-Za-z0-9_-]+)"
        r"(?:\s*,\s*rev:\s*[\"']?([^\"'\s,]*)[\"']?)?"
    )

    def test_files(self, root):
        found = list(super().test_files(root))
        for directory in self.test_dirs:
            base = os.path.join(root, directory)
            for current, _, names in os.walk(base):
                for name in sorted(names):
                    if name.startswith("test_") and name.endswith(".py"):
                        path = os.path.join(current, name)
                        if path not in found:
                            found.append(path)
        return sorted(found)

    def test_command(self):
        return [sys.executable, "-m", "unittest", "discover", "-s", "tests", "-t", "."]

    def ci_steps(self, root):
        return [
            "      - uses: actions/setup-python@v5",
            "        with:",
            f"          python-version: '{sys.version_info.major}."
            f"{sys.version_info.minor}'",
        ]

    def exports(self, root, modules):
        if not modules:
            return {}
        script = (
            "import importlib, inspect, sys\n"
            f"for name in {list(modules)!r}:\n"
            "    try:\n"
            "        mod = importlib.import_module(name)\n"
            "    except Exception:\n"
            f"        print('{EXPORT_MARK}' + name + '|__missing__')\n"
            "        continue\n"
            "    names = getattr(mod, '__all__', None)\n"
            "    if names is None:\n"
            "        names = [n for n in dir(mod) if not n.startswith('_')]\n"
            "    out = []\n"
            "    for n in names:\n"
            "        obj = getattr(mod, n, None)\n"
            "        out.append(n)\n"
            "        if callable(obj):\n"
            "            try:\n"
            "                params = inspect.signature(obj).parameters\n"
            "            except (TypeError, ValueError):\n"
            "                continue\n"
            "            count = len([p for p in params.values()\n"
            "                         if p.kind in (p.POSITIONAL_ONLY, p.POSITIONAL_OR_KEYWORD)])\n"
            "            out.append(n + '/' + str(count))\n"
            f"    print('{EXPORT_MARK}' + name + '|' + ','.join(out))\n"
        )
        proc = subprocess.run(
            [sys.executable, "-c", script],
            cwd=root, capture_output=True, text=True,
            env={**os.environ, "PYTHONPATH": root + os.pathsep + os.environ.get("PYTHONPATH", "")},
        )
        return parse_export_output(proc, modules)


def parse_export_output(proc, modules):
    out = {}
    for line in proc.stdout.splitlines():
        if not line.startswith(EXPORT_MARK):
            continue
        name, _, body = line[len(EXPORT_MARK):].partition("|")
        out[name] = None if body == "__missing__" else {
            item for item in body.split(",") if item
        }
    for name in modules:
        if name not in out:
            out[name] = None
    if proc.returncode != 0 and not any(out.values()):
        out["__error__"] = (proc.stderr or proc.stdout).strip()[:400]
    return out


ADAPTERS = (ElixirAdapter, PythonAdapter)


def detect_adapter(root):
    for adapter in ADAPTERS:
        if adapter.detect(root):
            return adapter()
    return None


# ─────────────────────────────────────────────────────────────────────────────
# Project
# ─────────────────────────────────────────────────────────────────────────────

class Problem:
    def __init__(self, check, message, where=None, line=None):
        self.check = check
        self.message = message
        self.where = where
        self.line = line

    def render(self):
        place = self.where or ""
        if place and self.line:
            place = f"{place}:{self.line}"
        return f"  {place}  {self.message}".rstrip() if place else f"  {self.message}"

    def as_dict(self):
        return {"check": self.check, "message": self.message,
                "file": self.where, "line": self.line}


class Project:
    def __init__(self, root):
        self.root = root
        self.keel = os.path.join(root, "keel")
        self.git = Git(root)
        self.adapter = detect_adapter(root)
        self.steps = {}
        self.contracts = {}
        self.decisions = {}
        self.broken = []
        # On CI the head is detached and git cannot name the branch — there the
        # name arrives in a flag.
        self.branch_override = None
        self._load()

    @property
    def branch(self):
        return self.branch_override or self.git.branch

    def _load(self):
        for kind, folder, cls in (
            ("steps", "steps", Step),
            ("contracts", "contracts", Contract),
            ("decisions", "decisions", Doc),
        ):
            target = getattr(self, kind if kind != "steps" else "steps")
            base = os.path.join(self.keel, folder)
            if not os.path.isdir(base):
                continue
            for name in sorted(os.listdir(base)):
                if not name.endswith(".md"):
                    continue
                path = os.path.join(base, name)
                doc = cls(path, self.root)
                if doc.error and kind != "decisions":
                    self.broken.append(doc)
                target[doc.slug] = doc

    @property
    def ready(self):
        return os.path.isdir(self.keel)

    def step_for_branch(self, branch=None):
        branch = branch or self.branch
        if not branch or branch in ("HEAD", self.git.main_short):
            return None
        name = branch.split("/", 1)[1] if branch.startswith("plan/") else branch
        return self.steps.get(name)

    def is_plan_branch(self, branch=None):
        return (branch or self.branch or "").startswith("plan/")

    def transform_state(self, step):
        """{transform -> (commit sha or None, {files of that commit})}."""
        base = self.git.merge_base(self.git.main_branch)
        found = {}
        for sha, message, files in self.git.commits_since(base):
            for slug in step.transforms:
                if slug in message and slug not in found:
                    found[slug] = (sha, files)
        return {slug: found.get(slug, (None, set())) for slug in step.transforms}


def find_root(start):
    current = os.path.abspath(start)
    while True:
        if os.path.isdir(os.path.join(current, "keel", "steps")):
            return current
        if os.path.isdir(os.path.join(current, ".git")):
            return current
        parent = os.path.dirname(current)
        if parent == current:
            return os.path.abspath(start)
        current = parent


# ─────────────────────────────────────────────────────────────────────────────
# The six checks
# ─────────────────────────────────────────────────────────────────────────────

CHECK_NAMES = {
    1: "посилання ведуть кудись",
    2: "depends_on без циклів",
    3: "редакції контрактів збіглися",
    4: "змінені файли збігаються з оголошеними",
    5: "у кожного сценарію зелений тест",
    6: "модулі експортують обіцяне",
    7: "імена в шапці збігаються із заголовками",
}

FAST_CHECKS = (1, 2, 3, 4, 7)
KEEL_DIR_PREFIX = "keel/"


def check_structure(project):
    return [Problem(0, doc.error, doc.rel) for doc in project.broken]


def check_refs(project):
    problems = []
    for step in project.steps.values():
        if step.error:
            continue
        for ref in step.depends_on:
            if ref.slug not in project.steps:
                problems.append(Problem(
                    1, f"depends_on показує на крок, якого немає: {ref.slug}",
                    step.rel, step.line_of(ref.slug)))
        for slug in step.scenarios:
            for ref in step.proves(slug):
                if ref.slug not in project.contracts:
                    problems.append(Problem(
                        1, f"сценарій {slug} доводить контракт, якого немає: {ref.slug}",
                        step.rel, step.line_of(ref.raw)))
        for slug in step.transforms:
            for name in step.transform_implements(slug):
                if name not in step.scenarios:
                    problems.append(Problem(
                        1, f"трансформа {slug} наближає сценарій, якого немає: {name}",
                        step.rel, step.line_of(name)))
            for ref in step.transform_contracts(slug):
                if ref.slug not in project.contracts:
                    problems.append(Problem(
                        1, f"трансформа {slug} реалізує контракт, якого немає: {ref.slug}",
                        step.rel, step.line_of(ref.raw)))

    for doc in list(project.steps.values()) + list(project.contracts.values()) + \
            list(project.decisions.values()):
        for match in LINK_RE.finditer(doc.body):
            target = match.group(1)
            if target.startswith(("http://", "https://")):
                continue
            resolved = os.path.normpath(os.path.join(os.path.dirname(doc.path), target))
            inside = os.path.relpath(resolved, project.keel).startswith("..")
            if inside or os.path.exists(resolved):
                continue
            problems.append(Problem(
                1, f"посилання нікуди не веде: {target}",
                doc.rel, doc.line_of(target)))
    return problems


def check_cycles(project):
    problems, state = [], {}

    def walk(slug, trail):
        if state.get(slug) == "done":
            return
        if state.get(slug) == "open":
            cycle = " → ".join(trail[trail.index(slug):] + [slug])
            problems.append(Problem(2, f"цикл у depends_on: {cycle}",
                                    project.steps[slug].rel))
            return
        state[slug] = "open"
        step = project.steps.get(slug)
        if step and not step.error:
            for ref in step.depends_on:
                if ref.slug in project.steps:
                    walk(ref.slug, trail + [slug])
        state[slug] = "done"

    for slug in sorted(project.steps):
        walk(slug, [])
    seen = set()
    return [p for p in problems if not (p.message in seen or seen.add(p.message))]


def contract_refs(step):
    """Everything in a step that leans on a contract: (who leans, reference)."""
    for slug in step.scenarios:
        for ref in step.proves(slug):
            yield f"сценарій {slug}", ref
    for slug in step.transforms:
        for ref in step.transform_contracts(slug):
            yield f"трансформа {slug}", ref


def scenario_tags(project):
    """(step, scenario, body, [(file, line, revision)]) — scenarios and their tags."""
    tags = project.adapter.tags(project.root) if project.adapter else {}
    for step in project.steps.values():
        if step.error:
            continue
        for slug in step.scenarios:
            body = step.scenario_body(slug)
            if body is None:
                continue  # check 7 catches this
            yield step, slug, body, tags.get(normalise_slug(slug), [])


def check_revisions(project):
    problems = []
    for step in project.steps.values():
        if step.error:
            continue
        for who, ref in contract_refs(step):
            contract = project.contracts.get(ref.slug)
            if contract is None or contract.error:
                continue
            if not ref.rev:
                problems.append(Problem(
                    3, f"{who} спирається на {ref.slug} без редакції; "
                       f"зараз {contract.revision}",
                    step.rel, step.line_of(ref.raw)))
            elif not contract.rev_ok(ref.rev):
                problems.append(Problem(
                    3, f"{who} тримає редакцію {ref.slug}@{ref.rev}, "
                       f"а контракт зараз {contract.revision}",
                    step.rel, step.line_of(ref.raw)))
    return problems


def check_scope(project):
    if not project.git.available:
        return [Problem(4, "це не git-репозиторій — межі перевірити нічим")]
    branch = project.branch
    if branch == project.git.main_short:
        return []
    if not branch or branch == "HEAD":
        # Passing silently would be a green check where none ever ran.
        return [Problem(4, "HEAD відчеплений, імені гілки git не знає — "
                           "передай його прапорцем --branch")]
    base = project.git.merge_base(project.git.main_branch)
    changed = {
        name for name in project.git.changed_files(base)
        if not name.startswith(KEEL_DIR_PREFIX)
    }

    if project.is_plan_branch(branch):
        stray = sorted(changed)
        return [Problem(4, f"гілка плану чіпає код: {name}") for name in stray]

    step = project.step_for_branch(branch)
    if step is None:
        return [Problem(4, f"гілка {branch} не називається кроком — "
                           f"немає з чим звіряти межі")]
    if step.error:
        return []

    declared = set()
    for slug in step.transforms:
        declared.update(step.transform_files(slug))

    problems = []
    for name in sorted(changed - declared):
        problems.append(Problem(4, f"файл змінено, але не оголошено: {name}", step.rel))
    for name in sorted(declared - changed):
        problems.append(Problem(4, f"файл оголошено, але не змінено: {name}",
                                step.rel, step.line_of(name)))
    return problems


def check_scenarios(project, run_tests=True):
    steps = [step for step in project.steps.values() if not step.error and step.scenarios]
    if not steps:
        return []
    if project.adapter is None:
        return [Problem(5, "не знайшов, чим запускати тести: у корені немає "
                           "жодного з " + ", ".join(
                               item for cls in ADAPTERS for item in cls.marker))]

    problems = []
    for step, slug, body, found in scenario_tags(project):
        if not found:
            problems.append(Problem(
                5, f"сценарій {slug} не має тесту", step.rel,
                step.section_lines.get(f"scenario: {slug}")))
            continue
        for path, line, rev in found:
            if not rev:
                problems.append(Problem(
                    5, f"тест сценарію {slug} без редакції; "
                       f"зараз {revision(body)}", path, line))
            elif not rev_matches(rev, body):
                problems.append(Problem(
                    5, f"тест тримає редакцію {slug}@{rev}, "
                       f"а сценарій зараз {revision(body)}", path, line))

    if run_tests:
        command = project.adapter.test_command()
        proc = subprocess.run(command, cwd=project.root, capture_output=True, text=True)
        if proc.returncode != 0:
            tail = (proc.stdout or proc.stderr).strip().splitlines()[-12:]
            problems.append(Problem(
                5, "тести червоні (" + " ".join(command) + "):\n"
                + "\n".join("      " + line for line in tail)))
    return problems


def check_exports(project):
    contracts = [c for c in project.contracts.values()
                 if not c.error and c.module and c.exports]
    if not contracts:
        return []
    if project.adapter is None:
        return [Problem(6, "не знайшов адаптера мови — експорти перевірити нічим")]

    modules = sorted({c.module for c in contracts})
    actual = project.adapter.exports(project.root, modules)
    problems = []
    if actual.get("__error__"):
        problems.append(Problem(6, "модулі не зібралися:\n      " + actual["__error__"]))
    for contract in contracts:
        have = actual.get(contract.module)
        if have is None:
            problems.append(Problem(
                6, f"модуля немає або він не зібрався: {contract.module}",
                contract.rel, contract.line_of("module")))
            continue
        for promised in contract.exports:
            if promised not in have:
                problems.append(Problem(
                    6, f"{contract.module} не експортує обіцяне: {promised}",
                    contract.rel, contract.line_of(promised)))
    return problems


def check_headings(project):
    problems = []
    for step in project.steps.values():
        if step.error:
            continue
        for kind, declared in (("scenario", step.scenarios), ("transform", step.transforms)):
            in_body = set(step.named_sections(kind))
            in_head = set(declared)
            for slug in sorted(in_head - in_body):
                problems.append(Problem(
                    7, f"у шапці є {kind} {slug}, у тілі секції немає",
                    step.rel, step.line_of(slug)))
            for slug in sorted(in_body - in_head):
                problems.append(Problem(
                    7, f"у тілі є ## {kind}: {slug}, у шапці його немає",
                    step.rel, step.section_lines.get(f"{kind}: {slug}")))
    return problems


def run_checks(project, only=None, run_tests=True):
    only = set(only or CHECK_NAMES)
    results = {}
    structural = check_structure(project)
    runners = {
        1: lambda: check_refs(project),
        2: lambda: check_cycles(project),
        3: lambda: check_revisions(project),
        4: lambda: check_scope(project),
        5: lambda: check_scenarios(project, run_tests=run_tests),
        6: lambda: check_exports(project),
        7: lambda: check_headings(project),
    }
    for number in sorted(runners):
        results[number] = runners[number]() if number in only else None
    return structural, results


# ─────────────────────────────────────────────────────────────────────────────
# Commands
# ─────────────────────────────────────────────────────────────────────────────

STEP_SKELETON = """---
depends_on: []

scenarios:
  # <slug>: {{proves: <contract>@<rev>}}

transforms:
  # <slug>:
  #   implements: [<scenario>]
  #   contracts:  [<contract>@<rev>]
  #   files:      [<шлях/до/файлу>]
---

## Навіщо

{slug}: навіщо цей крок і чого без нього бракує.

## scenario: <slug>

**Given** ...,
**When** ...,
**Then** ...

## transform: <slug>

Що робить.

Межі: чого не робить.
"""

CONTRACT_SKELETON = """---
module: <Module.Name>
exports: []
---

Що цей модуль обіцяє іншому коду.
"""


def cmd_new(project, args):
    kind, slug = args.kind, args.slug
    clean = normalise_slug(slug)
    if not clean:
        fail(f"поганий слаг: {slug!r}")

    if kind == "step":
        folder = os.path.join(project.keel, "steps")
        numbers = [int(m.group(1)) for name in os.listdir(folder)
                   if (m := re.match(r"(\d{4})-", name))] if os.path.isdir(folder) else []
        number = max(numbers, default=0) + 1
        name = f"{number:04d}-{clean}.md"
        text = STEP_SKELETON.format(slug=clean)
    else:
        folder = os.path.join(project.keel, "contracts")
        name = f"{clean}.md"
        text = CONTRACT_SKELETON

    path = os.path.join(folder, name)
    if os.path.exists(path):
        fail(f"вже є: {os.path.relpath(path, project.root)}")
    os.makedirs(folder, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    print(os.path.relpath(path, project.root))
    return 0


def cmd_plan(project, args):
    steps = ([project.steps[args.step]] if args.step and args.step in project.steps
             else [project.step_for_branch()] if not args.step and project.step_for_branch()
             else list(project.steps.values()))
    if args.step and args.step not in project.steps:
        fail(f"кроку немає: {args.step}")
    steps = [step for step in steps if step]

    problems = []
    for step in steps:
        if step.error:
            problems.append(Problem(0, step.error, step.rel))
            continue
        if not step.why.strip() or step.why.strip().startswith(f"{step.slug}: навіщо"):
            problems.append(Problem(0, "секція «Навіщо» порожня", step.rel))
        if not step.scenarios:
            problems.append(Problem(0, "жодного сценарію", step.rel))
        if not step.transforms:
            problems.append(Problem(0, "жодної трансформи", step.rel))

        implemented = set()
        for slug in step.transforms:
            implemented.update(step.transform_implements(slug))
            if not step.transform_files(slug):
                problems.append(Problem(
                    0, f"трансформа {slug} не оголосила файлів", step.rel,
                    step.line_of(slug)))
            if not step.transform_implements(slug):
                problems.append(Problem(
                    0, f"трансформа {slug} не наближає жодного сценарію", step.rel,
                    step.line_of(slug)))
            if not (step.transform_body(slug) or "").strip():
                problems.append(Problem(
                    0, f"трансформа {slug} без тіла: що робить і де межі", step.rel))
        for slug in step.scenarios:
            if not step.proves(slug):
                problems.append(Problem(
                    0, f"сценарій {slug} не має proves", step.rel, step.line_of(slug)))
            if slug not in implemented:
                problems.append(Problem(
                    0, f"сценарій {slug} не наближає жодна трансформа", step.rel,
                    step.line_of(slug)))
            if not (step.scenario_body(slug) or "").strip():
                problems.append(Problem(
                    0, f"сценарій {slug} без тіла: given/when/then", step.rel))

    problems += check_headings(project) if not args.step else [
        p for p in check_headings(project) if p.where in {s.rel for s in steps}]
    problems += [p for p in check_refs(project) if p.where in {s.rel for s in steps}]

    names = ", ".join(step.slug for step in steps) or "нічого"
    if not problems:
        print(f"план повний: {names}")
        return 0
    print(f"плану бракує ({names}):\n")
    for problem in problems:
        print(problem.render())
    print(f"\nвсього: {len(problems)}")
    return 1


def cmd_check(project, args):
    only = FAST_CHECKS if args.fast else None
    structural, results = run_checks(project, only, run_tests=not args.no_tests)

    if args.json:
        payload = {
            "ok": not structural and not any(results.get(n) for n in results),
            "structure": [p.as_dict() for p in structural],
            "checks": {
                str(number): {
                    "name": CHECK_NAMES[number],
                    "run": results[number] is not None,
                    "problems": [p.as_dict() for p in (results[number] or [])],
                }
                for number in sorted(results)
            },
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0 if payload["ok"] else 1

    total = len(structural)
    if structural:
        print("✗ документи не читаються")
        for problem in structural:
            print(problem.render())
        print()

    for number in sorted(results):
        problems = results[number]
        if problems is None:
            print(f"– {number}. {CHECK_NAMES[number]} (не запускалась)")
            continue
        total += len(problems)
        if not problems:
            print(f"✓ {number}. {CHECK_NAMES[number]}")
            continue
        print(f"✗ {number}. {CHECK_NAMES[number]}")
        for problem in problems:
            print(problem.render())
    print()
    print("чисто" if total == 0 else f"проблем: {total}")
    return 0 if total == 0 else 1


def next_transform(project, step):
    state = project.transform_state(step)
    for slug in step.transforms:
        if state[slug][0] is None:
            return slug, state
    return None, state


def cmd_next(project, args):
    step = project.step_for_branch()
    branch = project.branch
    if step is None:
        message = (f"гілка {branch} не називається кроком. "
                   f"Робота йде в гілці з іменем кроку, план — у plan/<крок>.")
        return emit_next_error(args, message)
    if project.is_plan_branch(branch):
        return emit_next_error(args, "це гілка плану: тут пишеться крок, а не код. "
                                     "keel plan скаже, чого бракує.")
    if not project.git.file_in_branch(project.git.main_branch, step.rel):
        return emit_next_error(
            args, f"крок {step.slug} ще не в гілці {project.git.main_branch}: "
                  f"план не схвалений, роботи немає.")
    if step.error:
        return emit_next_error(args, f"{step.rel}: {step.error}")

    slug, state = next_transform(project, step)
    if slug is None:
        message = (f"усі трансформи кроку {step.slug} закриті коммітами. "
                   f"Далі: keel check, потім PR.")
        return emit_next_error(args, message, code=0)

    contracts = []
    for ref in step.transform_contracts(slug):
        contract = project.contracts.get(ref.slug)
        contracts.append({
            "slug": ref.slug,
            "rev": ref.rev,
            "rev_ok": bool(contract and not contract.error and contract.rev_ok(ref.rev)),
            "rev_now": contract.revision if contract and not contract.error else None,
            "module": contract.module if isinstance(contract, Contract) else None,
            "exports": contract.exports if isinstance(contract, Contract) else [],
            "body": contract.body.strip() if contract else None,
        })

    scenarios = []
    for name in step.transform_implements(slug):
        body = step.scenario_body(name)
        scenarios.append({
            "slug": name,
            "rev": step.scenario_revision(name),
            "proves": [ref.raw for ref in step.proves(name)],
            "body": (body or "").strip(),
        })

    package = {
        "step": {"id": step.slug, "file": step.rel, "why": step.why.strip()},
        "transform": {
            "slug": slug,
            "body": (step.transform_body(slug) or "").strip(),
            "files": step.transform_files(slug),
        },
        "scenarios": scenarios,
        "contracts": contracts,
        "done": [name for name, (sha, _) in state.items() if sha],
        "left": [name for name in step.transforms
                 if state[name][0] is None and name != slug],
        "commit": f"{slug}: <що зроблено>",
        "tag_hint": [
            {"scenario": item["slug"], "rev": item["rev"]} for item in scenarios
        ],
    }

    if args.json:
        print(json.dumps(package, ensure_ascii=False, indent=2))
    else:
        print(render_next(package))
    return 0


def emit_next_error(args, message, code=1):
    if args.json:
        print(json.dumps({"error": message, "done": code == 0}, ensure_ascii=False, indent=2))
    else:
        print(message)
    return code


def render_next(package):
    step, transform = package["step"], package["transform"]
    out = [f"# {transform['slug']}", ""]
    out.append(f"Крок {step['id']} · {step['file']}")
    if package["done"]:
        out.append(f"Закрито: {', '.join(package['done'])}")
    if package["left"]:
        out.append(f"Після цієї: {', '.join(package['left'])}")
    out.append("")
    if step["why"]:
        out += ["## Навіщо крок", "", step["why"], ""]
    if transform["body"]:
        out += ["## Ця трансформа", "", transform["body"], ""]
    out += ["## Файли, і тільки вони", ""]
    out += [f"- {name}" for name in transform["files"]] or ["- (не оголошено — план неповний)"]
    out.append("")

    if package["scenarios"]:
        out += ["## Сценарії, які вона наближає", ""]
        for item in package["scenarios"]:
            out.append(f"### {item['slug']}")
            out.append("")
            out.append(item["body"] or "(тіла немає)")
            out.append("")
            out.append(f"Тег тесту: `proves: :{item['slug'].replace('-', '_')}, "
                       f"rev: \"{item['rev']}\"`")
            out.append("")

    if package["contracts"]:
        out += ["## Контракти, на які вона спирається", ""]
        for item in package["contracts"]:
            head = f"### {item['slug']}"
            if item["module"]:
                head += f" — `{item['module']}`"
            out.append(head)
            out.append("")
            if item["exports"]:
                out.append(f"Експортує: {', '.join(item['exports'])}")
                out.append("")
            out.append(item["body"] or "(контракту немає)")
            out.append("")
            if not item["rev_ok"]:
                out.append(f"⚠ редакція в кроці {item['rev']}, "
                           f"а контракт зараз {item['rev_now']} — спершу keel rev")
                out.append("")

    out += ["## Комміт", "", f"    {package['commit']}", ""]
    out.append("Слаг трансформи в повідомленні — єдиний звʼязок роботи з планом.")
    return "\n".join(out)


INIT_DIRS = ("keel/steps", "keel/contracts", "keel/decisions")
AGENTS_START = "<!-- keel:start -->"
AGENTS_END = "<!-- keel:end -->"
VENDORED = "keel/keel.py"
CI_FILE = ".github/workflows/keel.yml"
# Довідники їдуть копіями: AGENTS.md показує на них, а показувати можна лише
# на те, що лежить у цьому ж репозиторії.
REFERENCES = ("KEEL.md", "QUALITY.md")

AGENTS_BLOCK = """{start}
## Keel

Методика цього проєкту: три типи документів — крок, контракт, рішення — і шість
перевірок. Кроки лежать у `keel/steps/`, контракти в `keel/contracts/`.

{principles}

Дві команди:

- `python3 {tool} next` — що робити далі: трансформа, її файли й межі,
  сценарії, які вона наближає, тіла контрактів, на які спирається.
- `python3 {tool} check` — що не так зараз. Перед коммітом і перед PR.

Два довідники — відкривай, коли не ясно:

- `keel/KEEL.md` — формат: що йде в шапку кроку, як влаштовані редакції,
  що саме перевіряє кожна з шести перевірок.
- `keel/QUALITY.md` — сорок розрізів якості. Проходяться раз на крок, там,
  де пишуться сценарії.

Цей блок породжений; правки між маркерами затре наступне оновлення.
{end}"""

CI_TEMPLATE = """name: keel
# Породжено `keel init`. Правки затре наступне оновлення.
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # перевірка меж звіряє гілку з main
{setup}      - run: python3 {tool} check --branch "${{{{ github.head_ref || github.ref_name }}}}"
"""


def home():
    return os.path.dirname(os.path.abspath(__file__))


def principles_lines():
    """The seven statements from PRINCIPLES.md — headings, without bodies."""
    path = os.path.join(home(), "PRINCIPLES.md")
    if not os.path.exists(path):
        return None
    found = re.findall(r"^##\s+(\d+)\.\s+(.+?)\s*$", read_text(path), re.M)
    return [f"{number}. {title}." for number, title in found] or None


def cmd_init(project, args):
    principles = principles_lines()
    sources = {name: os.path.join(home(), name) for name in REFERENCES}
    if principles is None or not all(map(os.path.exists, sources.values())):
        fail("PRINCIPLES.md, KEEL.md і QUALITY.md поруч не знайшлись: "
             "init запускають із репозиторію методики")

    done = []
    for folder in INIT_DIRS:
        os.makedirs(os.path.join(project.root, folder), exist_ok=True)
    done.append("keel/steps, keel/contracts, keel/decisions")

    source = os.path.abspath(__file__)
    target = os.path.join(project.root, VENDORED)
    if os.path.abspath(target) != source:
        write_if_changed(target, read_text(source), done, VENDORED)
    for name, path in sources.items():
        write_if_changed(os.path.join(project.root, "keel", name),
                         read_text(path), done, f"keel/{name}")

    block = AGENTS_BLOCK.format(
        start=AGENTS_START, end=AGENTS_END, tool=VENDORED,
        principles="\n".join(principles))
    if update_agents(os.path.join(project.root, "AGENTS.md"), block):
        done.append("AGENTS.md (блок між маркерами)")

    setup = project.adapter.ci_steps(project.root) if project.adapter else []
    write_if_changed(
        os.path.join(project.root, CI_FILE),
        CI_TEMPLATE.format(tool=VENDORED,
                           setup="".join(line + "\n" for line in setup)),
        done, CI_FILE)

    for line in done:
        print(f"  {line}")
    return cmd_hooks(project, args)


def write_if_changed(path, text, done, label):
    if os.path.exists(path) and read_text(path) == text:
        return False
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    done.append(label)
    return True


def update_agents(path, block):
    """Add the block to AGENTS.md without touching the rest of the file."""
    old = read_text(path) if os.path.exists(path) else ""
    if AGENTS_START in old and AGENTS_END in old:
        head, _, rest = old.partition(AGENTS_START)
        _, _, tail = rest.partition(AGENTS_END)
        new = head + block + tail
    elif old.strip():
        new = old.rstrip("\n") + "\n\n" + block + "\n"
    else:
        new = block + "\n"
    if new == old:
        return False
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(new)
    return True


# Fast on commit, slow on push: the agent commits often and must not wait
# minutes, and red will not reach the main branch either way.
HOOKS = {
    "pre-commit": ("check", "--fast"),
    "pre-push": ("check",),
}

HOOK_MARK = "# keel hook:"

HOOK_SCRIPT = """#!/bin/sh
{mark} {name}. Породжено `keel hooks --install`.
# Правки цього файлу затре наступна установка.
set -eu

run() {{
  case "$1" in
    *.py) exec python3 "$1" {args} ;;
    *)    exec "$1" {args} ;;
  esac
}}

# Інструмент шукається в такому порядку: змінна KEEL, PATH, копія в проєкті,
# і аж тоді шлях, який був у машини під час установки.
if [ -n "${{KEEL:-}}" ] && [ -f "${{KEEL}}" ]; then run "${{KEEL}}"; fi

tool=$(command -v keel 2>/dev/null || true)
if [ -n "$tool" ]; then run "$tool"; fi

root=$(git rev-parse --show-toplevel)
if [ -f "$root/keel/keel.py" ]; then run "$root/keel/keel.py"; fi

if [ -f "{baked}" ]; then run "{baked}"; fi

echo "keel: інструмента не знайшов. Постав KEEL=/шлях/до/keel.py" >&2
exit 1
"""


def hook_script(name, baked):
    return HOOK_SCRIPT.format(
        mark=HOOK_MARK, name=name, baked=baked, args=" ".join(HOOKS[name]))


def cmd_hooks(project, args):
    git_dir = project.git.out("rev-parse", "--absolute-git-dir")
    if not git_dir:
        fail("це не git-репозиторій — хуки нема куди класти")
    folder = os.path.join(git_dir, "hooks")
    baked = os.path.abspath(__file__)

    problems, missing = 0, 0
    for name in HOOKS:
        path = os.path.join(folder, name)
        present = os.path.exists(path)
        mine = present and HOOK_MARK in read_text(path)

        if not args.install:
            state = "наш" if mine else ("чужий" if present else "немає")
            missing += 0 if mine else 1
            print(f"  {name}: {state}")
            continue
        if present and not mine and not args.force:
            print(f"  {name}: чужий хук, не чіпаю (--force щоб перезаписати)")
            problems += 1
            continue
        os.makedirs(folder, exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(hook_script(name, baked))
        os.chmod(path, 0o755)
        print(f"  {name}: {' '.join(('keel',) + HOOKS[name])}")

    if not args.install:
        print("\nkeel hooks --install щоб поставити" if missing else "\nстоять обидва")
        return 0
    return 1 if problems else 0


def read_text(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return ""


def cmd_rev(project, args):
    """Show revisions that have drifted apart; --write records the new ones."""
    edits = {}   # path -> [(old, new)]
    report = []

    for step in project.steps.values():
        if step.error:
            continue
        for who, ref in contract_refs(step):
            contract = project.contracts.get(ref.slug)
            if contract is None or contract.error:
                continue
            if ref.rev and contract.rev_ok(ref.rev):
                continue
            fresh = contract.revision
            report.append((step.rel, who, f"{ref.slug}@{ref.rev or '—'}",
                           f"{ref.slug}@{fresh}"))
            edits.setdefault(step.path, []).append((ref.raw, f"{ref.slug}@{fresh}"))

    for _, slug, body, found in scenario_tags(project):
        fresh = revision(body)
        for path, line, rev in found:
            if rev and rev_matches(rev, body):
                continue
            report.append((f"{path}:{line}", f"тест {slug}", rev or "—", fresh))
            edits.setdefault(os.path.join(project.root, path), []).append(
                (("TAG", slug), fresh))

    if not report:
        print("усі редакції збігаються")
        return 0

    for where, who, was, now in report:
        print(f"  {where}  {who}: {was} → {now}")

    if not args.write:
        print(f"\nрозійшлося: {len(report)}. keel rev --write впише нові — "
              f"після того, як перечитаєш текст, на який спираєшся.")
        return 1

    for path, changes in edits.items():
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        for old, new in changes:
            if isinstance(old, tuple):
                text = rewrite_tag(text, old[1], new)
            else:
                text = text.replace(old, new)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    print(f"\nвписано: {len(report)}")
    return 0


def rewrite_tag(text, slug, fresh):
    """Write a fresh revision into a test tag, in whichever form is already there."""
    atom = slug.replace("-", "_")
    elixir = re.compile(
        rf"@tag\s+proves:\s*:({re.escape(atom)})"
        rf"(?:\s*,\s*rev:\s*[\"'][^\"']*[\"'])?"
    )
    text = elixir.sub(lambda m: f'@tag proves: :{m.group(1)}, rev: "{fresh}"', text)
    python = re.compile(
        rf"#\s*proves:\s*({re.escape(slug)}|{re.escape(atom)})"
        rf"(?:\s*,\s*rev:\s*[\"']?[^\"'\s,]*[\"']?)?"
    )
    return python.sub(lambda m: f'# proves: {m.group(1)}, rev: "{fresh}"', text)


# ─────────────────────────────────────────────────────────────────────────────
# Entry point
# ─────────────────────────────────────────────────────────────────────────────

def fail(message, code=2):
    print(message, file=sys.stderr)
    raise SystemExit(code)


def build_parser():
    parser = argparse.ArgumentParser(
        prog="keel", description="Keel: три типи документів, шість перевірок.")
    parser.add_argument("--version", action="version", version=VERSION)
    parser.add_argument("-C", dest="chdir", metavar="ТЕКА",
                        help="працювати в цій теці")
    sub = parser.add_subparsers(dest="command", required=True)

    new = sub.add_parser("new", help="каркас кроку або контракту")
    new.add_argument("kind", choices=("step", "contract"))
    new.add_argument("slug")

    plan = sub.add_parser("plan", help="повнота плану")
    plan.add_argument("step", nargs="?", help="крок; без нього — крок гілки")

    check = sub.add_parser("check", help="шість перевірок")
    check.add_argument("--fast", action="store_true",
                       help="лише 1, 2, 3, 4, 7 — те, що вішається на pre-commit")
    check.add_argument("--no-tests", action="store_true", help="не запускати тести")
    check.add_argument("--branch", metavar="ІМʼЯ",
                       help="імʼя гілки, коли git його не знає — на CI")
    check.add_argument("--json", action="store_true")

    nxt = sub.add_parser("next", help="пакет наступної дії")
    nxt.add_argument("--json", action="store_true")

    rev = sub.add_parser("rev", help="редакції, які розійшлися")
    rev.add_argument("--write", action="store_true", help="вписати нові редакції")

    hooks = sub.add_parser("hooks", help="хуки git: pre-commit і pre-push")
    hooks.add_argument("--install", action="store_true")
    hooks.add_argument("--force", action="store_true", help="перезаписати чужий хук")

    init = sub.add_parser("init", help="поставити Keel у проєкт")
    init.add_argument("--force", action="store_true", help="перезаписати чужий хук")
    init.set_defaults(install=True)

    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    start = args.chdir or os.getcwd()
    if not os.path.isdir(start):
        fail(f"теки немає: {start}")
    root = find_root(start)
    project = Project(root)
    project.branch_override = getattr(args, "branch", None)

    if args.command not in ("new", "init") and not project.ready:
        fail(f"у {root} немає теки keel/ — тут Keel не поставлений")

    handlers = {"new": cmd_new, "plan": cmd_plan, "check": cmd_check,
                "next": cmd_next, "rev": cmd_rev, "hooks": cmd_hooks,
                "init": cmd_init}
    return handlers[args.command](project, args)


if __name__ == "__main__":
    sys.exit(main())
