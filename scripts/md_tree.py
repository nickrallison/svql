#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///


import argparse
import textwrap
import fnmatch
import os
import sys
import argparse
from pathlib import Path
from typing import Iterable, List, Sequence, Set, Dict

# ----------------------------------------------------------------------
# Mapping from file‑extension to fenced‑code language
# ----------------------------------------------------------------------
EXTENSION_MAP = {
    ".v": "verilog",
    ".sv": "systemverilog",
    ".vh": "verilog",
    ".vhd": "vhdl",
    ".vhdl": "vhdl",
    ".svh": "systemverilog",
    ".c": "c",
    ".h": "c",
    ".cpp": "cpp",
    ".hpp": "cpp",
    ".py": "python",
    ".js": "javascript",
    ".ts": "typescript",
    ".rb": "ruby",
    ".go": "go",
    ".rs": "rust",
    ".java": "java",
}


def _guess_language(path: Path) -> str:
    """Return the markdown fence language for *path*."""
    return EXTENSION_MAP.get(path.suffix.lower(), "text")


def _iter_files(
    root: Path,
    includes: Sequence[str] | None,
    excludes: Sequence[str] | None,
) -> Iterable[Path]:
    """
    Yield files under *root* that match the include patterns and do **not**
    match any exclude pattern.
    """
    if includes:
        include_set = set(includes)
    else:
        include_set = {"**"}  # match everything

    exclude_set = set(excludes or ())

    for dirpath, dirnames, filenames in os.walk(root):
        rel_dir = Path(dirpath).relative_to(root)

        # ------------------------------------------------------------------
        # Prune directories early if they are excluded (helps speed)
        # ------------------------------------------------------------------
        dirnames[:] = [
            d
            for d in dirnames
            if not any(
                fnmatch.fnmatch(str(rel_dir / d), pat) for pat in exclude_set
            )
        ]

        for fname in filenames:
            rel_path = rel_dir / fname
            # Apply include patterns
            if not any(fnmatch.fnmatch(str(rel_path), pat) for pat in include_set):
                continue
            # Apply exclude patterns
            if any(fnmatch.fnmatch(str(rel_path), pat) for pat in exclude_set):
                continue
            yield root / rel_path


# ----------------------------------------------------------------------
# SECTION‑HEADER HELPERS
# ----------------------------------------------------------------------
def _parse_section_map(raw: Sequence[str]) -> Dict[Path, str]:
    """
    Convert a list of ``"path=title"`` strings into a dict
    ``{Path('path'): 'title'}``.  The *path* is stored **as a Path object**
    relative to the root, without a trailing slash.
    """
    mapping: Dict[Path, str] = {}
    for item in raw:
        if "=" not in item:
            raise argparse.ArgumentTypeError(
                f'Invalid --section value: "{item}". Expected "path=title".'
            )
        p, title = item.split("=", 1)
        # Normalise the path – remove a possible trailing slash
        mapping[Path(p.rstrip("/"))] = title
    return mapping


def _emit_hierarchical_headers(
    rel_path: Path,
    printed_dirs: Set[Path],
    base_level: int,
    out: List[str],
    section_map: Dict[Path, str],
) -> None:
    """
    Emit markdown headers for each directory component of *rel_path* that has
    not yet been printed.

    * ``rel_path`` – path **relative** to the root (e.g. ``a/b/c.v``)
    * ``printed_dirs`` – a set that remembers which directories have already
      been emitted, so we do not repeat them.
    * ``base_level`` – the header level for the *first* directory (default 2 → `##`).
    * ``out`` – the list that accumulates the markdown lines.
    * ``section_map`` – optional user‑provided mapping ``Path → title``.
    """
    parts = rel_path.parent.parts  # tuple of directory names, may be empty
    for depth, part in enumerate(parts, start=1):
        dir_path = Path(*parts[:depth])          # e.g. Path('a/b')
        if dir_path in printed_dirs:
            continue
        printed_dirs.add(dir_path)

        # Use the user‑provided title if it exists, otherwise fall back to the
        # directory name.
        title = section_map.get(dir_path, part)

        header_level = base_level + depth - 1
        out.append(f"{'#' * header_level} {title}")


# ----------------------------------------------------------------------
# MARKDOWN GENERATOR
# ----------------------------------------------------------------------
def generate_markdown(
    root: Path,
    includes: Sequence[str] | None = None,
    excludes: Sequence[str] | None = None,
    *,
    sort: bool = True,
    header_base_level: int = 2,
    section_map: Dict[Path, str] | None = None,
) -> str:
    """
    Return a Markdown string that contains the contents of all matching files.

    Parameters
    ----------
    root
        Directory that acts as the reference point for all include / exclude
        patterns.
    includes, excludes
        See :func:`_iter_files`.
    sort
        If ``True`` (default) the file list is sorted alphabetically before
        rendering – this makes the output deterministic.
    header_base_level
        Header depth for the *first* directory level (``2`` → ``##``).  Deeper
        directories get progressively deeper headers.
    section_map
        Optional mapping ``Path → title`` that lets the user rename any directory
        (or arbitrary path) in the hierarchy.
    """
    files = list(_iter_files(root, includes, excludes))
    if sort:
        files.sort()

    lines: List[str] = []
    printed_dirs: Set[Path] = set()
    section_map = section_map or {}

    for f in files:
        rel = f.relative_to(root)

        # 1️⃣ Emit directory hierarchy (if any)
        if rel.parent != Path():
            _emit_hierarchical_headers(
                rel,
                printed_dirs,
                header_base_level,
                lines,
                section_map,
            )

        # 2️⃣ Emit the per‑file header (always one level deeper than the deepest
        #    directory header for that file)
        file_header_level = header_base_level + len(rel.parent.parts)
        lines.append(f"{'#' * file_header_level} {rel}")

        # 3️⃣ Emit the fenced code block
        language = _guess_language(f)
        lines.append(f"```{language}")
        try:
            # Read the file using its native newline handling.
            content = f.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            # Binary or non‑utf‑8 files – fall back to a safe representation.
            content = f.read_bytes().hex()
            lines.append("\n# (binary file – shown as hex)\n")

        content = "".join(['\t' + line for line in content.splitlines(keepends=True)])

        lines.append(content.rstrip("\n"))  # avoid extra blank line at end
        lines.append("```")
        lines.append("")  # blank line between sections

    return "\n".join(lines)


# ----------------------------------------------------------------------
# ARGUMENT PARSING
# ----------------------------------------------------------------------
def _parse_args(argv: List[str] | None = None) -> argparse.Namespace:
    """
    Build the command‑line parser.

    The epilog contains a nicely formatted example.  We use a custom formatter
    that mixes ``ArgumentDefaultsHelpFormatter`` (so the “(default: …)” text is
    still shown) with ``RawDescriptionHelpFormatter`` (so our line breaks are
    preserved).
    """
    # ------------------------------------------------------------------
    # Custom formatter that keeps new‑lines *and* shows defaults
    # ------------------------------------------------------------------
    class RawDefaultsHelpFormatter(
        argparse.ArgumentDefaultsHelpFormatter,
        argparse.RawDescriptionHelpFormatter,
    ):
        pass

    parser = argparse.ArgumentParser(
        description=(
            "Generate a Markdown document that embeds source files with a "
            "user‑defined hierarchical header structure."
        ),
        formatter_class=RawDefaultsHelpFormatter,
        epilog=textwrap.dedent(
            """
            Example:
              ./md_tree.py \\
                  --root examples \\
                  --include "**/*.v" \\
                  --exclude "**/test_*" \\
                  --header-base-level 2 \\
                  --section "examples=Examples" \\
                  --section "examples/basic=Basic patterns" \\
                  --section "examples/basic/and=AND‑gate examples" \\
                  --output documentation.md

            The command above:
              • walks the *examples* directory,
              • includes only Verilog files,
              • skips any file whose name starts with `test_`,
              • starts the hierarchy at level ``##`` (``--header-base-level 2``),
              • renames three directories with custom titles, and
              • writes the generated Markdown to ``documentation.md``.
            """
        ),
    )

    # ------------------------------------------------------------------
    # The rest of the arguments stay exactly the same
    # ------------------------------------------------------------------
    parser.add_argument(
        "--root",
        type=Path,
        default=Path.cwd(),
        help="Root directory to search (all paths are shown relative to this).",
    )
    parser.add_argument(
        "--include",
        action="append",
        default=[],
        help=(
            "Glob pattern (relative to --root) to include. "
            "Can be given multiple times. If omitted, everything is included."
        ),
    )
    parser.add_argument(
        "--exclude",
        action="append",
        default=[],
        help=(
            "Glob pattern (relative to --root) to exclude. "
            "Can be given multiple times."
        ),
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Write the markdown to this file. If omitted, write to stdout.",
    )
    parser.add_argument(
        "--no-sort",
        dest="sort",
        action="store_false",
        help="Do not sort the file list (preserve discovery order).",
    )
    parser.add_argument(
        "--header-base-level",
        type=int,
        default=2,
        metavar="N",
        help=(
            "Header level for the first directory component (default: 2 → ##). "
            "Deeper directories get N+1, N+2, …"
        ),
    )
    parser.add_argument(
        "--section",
        action="append",
        default=[],
        metavar="PATH=TITLE",
        help=(
            "Define a custom title for a directory (or any path) relative to "
            "--root. The format is \"PATH=TITLE\" and the option can be used "
            "multiple times. Example: --section \"examples=Examples\" "
            "--section \"examples/basic=Basic\""
        ),
    )
    return parser.parse_args(argv)


def main(argv: List[str] | None = None) -> int:
    args = _parse_args(argv)

    # Build the mapping from the repeated --section arguments
    section_map = _parse_section_map(args.section)

    markdown = generate_markdown(
        root=args.root,
        includes=args.include or None,
        excludes=args.exclude or None,
        sort=args.sort,
        header_base_level=args.header_base_level,
        section_map=section_map,
    )

    if args.output:
        args.output.write_text(markdown, encoding="utf-8")
    else:
        sys.stdout.write(markdown)

    return 0


if __name__ == "__main__":
    sys.exit(main())