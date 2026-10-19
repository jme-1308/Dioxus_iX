#!/usr/bin/env python3
"""Check the Rust icon bundle against the published @siemens/ix-icons 3.7.0.

Development-only, stdlib-only, offline checker. It reads SVG/XML and JSON data;
it never installs or executes the package's JavaScript. Example:

    python3 tools/check_icon_catalog.py target/.../asset-ix-icons-3.7.0.tgz \
        --registry target/.../asset-registry.json --report target/.../asset-report.json

An extracted package directory is also accepted. Only a tarball can establish
the pinned registry archive integrity; directory reports explicitly say otherwise.
This checks data framing and representative normalized geometry, not browser
rendering equivalence, trademark clearance, or every source member's behavior.
"""

import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import re
import sys
import tarfile
import xml.etree.ElementTree as ET


PACKAGE_NAME = "@siemens/ix-icons"
VERSION = "3.7.0"
REGISTRY_URL = "https://registry.npmjs.org/@siemens%2fix-icons/3.7.0"
TARBALL_URL = "https://registry.npmjs.org/@siemens/ix-icons/-/ix-icons-3.7.0.tgz"
ARCHIVE_SHA256 = "09f50676e3cd42958210181b66f64ba8a4f79739fab969a38c02f8bf77a935a3"
ARCHIVE_INTEGRITY = (
    "sha512-1nk58wy/s+Snz0Fw1b80IM5fzz53wtDJ54yLmEfIX317LYQcp9Z0jU5V7MQvbiuJ0HLr"
    "PgwJe1kvTrn8z5S5Hg=="
)
EXCLUDED = {
    "bilibili-logo", "facebook-logo", "github-logo", "instagram-logo",
    "linkedin-logo", "mastodon-logo", "profisafe-logo", "reddit-logo",
    "telegram-logo", "threads-logo", "tiktok-logo", "viva-engage-logo",
    "wechat-logo", "weibo-logo", "x-logo", "youtube", "youtube-filled",
}
REPRESENTATIVES = (
    "aas-project-ax", "aas-project-generic-filled", "about", "check",
    "application-screen",
)
XML_WRAPPER = '<svg xmlns:xlink="http://www.w3.org/1999/xlink">{}</svg>'
SUPPORTED_TAGS = {"svg", "path", "g", "polygon", "rect", "mask", "defs", "use", "clipPath"}
MONOCHROME_FILL = {"none", "#000", "#000000", "black", "currentColor"}


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def local_name(name):
    return name.rsplit("}", 1)[-1]


class Package:
    def __init__(self, path):
        self.path = path
        self.archive_bytes = None
        self.archive = None
        if path.is_dir():
            self.root = path / "package" if (path / "package/package.json").is_file() else path
        else:
            self.archive_bytes = path.read_bytes()
            self.archive = tarfile.open(fileobj=io.BytesIO(self.archive_bytes), mode="r:gz")

    def read(self, name):
        if self.archive is None:
            return (self.root / name).read_bytes()
        member = self.archive.getmember("package/" + name)
        if not member.isfile():
            raise ValueError(f"Not a regular package file: {name}")
        return self.archive.extractfile(member).read()

    def svg_names(self):
        if self.archive is None:
            return {path.stem for path in (self.root / "svg").glob("*.svg")}
        return {
            Path(member.name).stem for member in self.archive.getmembers()
            if member.isfile() and re.fullmatch(r"package/svg/[^/]+\.svg", member.name)
        }


def parse_bundle(source):
    # Parse this data declaration only, not Rust test fixtures or runtime code.
    declaration = source.split('const ICONS: &[(&str, &str)] = &[', 1)[1]
    declaration = declaration.rsplit("];", 1)[0]
    entries = re.findall(r'\(\s*"([^"\n]+)",\s*r##"(.*?)"##,?\s*\)', declaration, re.S)
    if not entries or len({name for name, _ in entries}) != len(entries):
        raise ValueError("Missing or duplicate Rust icon tuples")
    if [name for name, _ in entries] != sorted(name for name, _ in entries):
        raise ValueError("Rust icon tuples are not sorted")
    return dict(entries)


def geometry_tree(element):
    """Preserve geometric values, hierarchy, rules and transforms.

    Normalize namespace spelling/attribute order and monochrome fill inheritance.
    Root framing is checked separately. Source descriptions are metadata.
    """
    attributes = []
    for key, value in element.attrib.items():
        if key == "fill":
            if value not in MONOCHROME_FILL:
                raise ValueError(f"Unexpected source color {value!r}")
            continue
        attributes.append((key, value))
    return [local_name(element.tag), sorted(attributes), [
        geometry_tree(child) for child in element if local_name(child.tag) != "desc"
    ]]


def check(args):
    errors = []
    package = Package(args.artifact)
    manifest = json.loads(package.read("package.json"))
    if (manifest.get("name"), manifest.get("version"), manifest.get("license")) != (
        PACKAGE_NAME, VERSION, "MIT"
    ):
        errors.append("Package name/version/license does not match the pinned MIT source")
    report = {
        "source": {"name": PACKAGE_NAME, "version": VERSION, "registry_url": REGISTRY_URL,
                   "tarball_url": TARBALL_URL},
        "archive_integrity_verified": False,
        "limitations": [
            "Registry integrity checks archive bytes, not publisher identity/signatures.",
            "Representative normalized geometry comparisons are not exact SVG byte comparisons.",
            "Static XML/framing checks do not prove browser rendering or legal/trademark clearance.",
        ],
    }
    if package.archive_bytes is not None:
        data = package.archive_bytes
        integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()
        report["archive_sha256"] = sha256(data)
        report["archive_integrity"] = integrity
        if sha256(data) != ARCHIVE_SHA256 or integrity != ARCHIVE_INTEGRITY:
            errors.append("Tarball does not match the pinned official 3.7.0 archive")
        else:
            report["archive_integrity_verified"] = True
        if args.registry:
            metadata = json.loads(args.registry.read_bytes())
            dist = metadata.get("dist", {})
            if (metadata.get("name"), metadata.get("version"), dist.get("integrity"),
                dist.get("tarball"), dist.get("shasum")) != (
                PACKAGE_NAME, VERSION, integrity, TARBALL_URL, hashlib.sha1(data).hexdigest()
            ):
                errors.append("Saved registry metadata disagrees with tarball bytes/source pin")
            report["registry_metadata_sha256"] = sha256(args.registry.read_bytes())
    else:
        report["limitations"].append("Extracted directory supplied: archive integrity was not verified.")

    license_bytes = package.read("LICENSE.md")
    license_text = license_bytes.decode("utf-8")
    notices = (args.repo / "THIRD_PARTY_NOTICES.md").read_text()
    icon_notice = notices.split("## Siemens iX icons 3.7.0", 1)[1].split("\n## ", 1)[0]
    retained_text = icon_notice.split("```text\n", 1)[1].split("\n```", 1)[0]
    if retained_text != license_text.rstrip("\n"):
        errors.append("Retained icon MIT notice differs from package/LICENSE.md")
    report["license"] = {
        "path": "package/LICENSE.md", "bytes": len(license_bytes),
        "sha256": sha256(license_bytes), "copyright": license_text.splitlines()[2],
        "text": license_text, "retained_notice_matches": retained_text == license_text.rstrip("\n"),
    }

    source_bytes = (args.repo / "src/icons.rs").read_bytes()
    bundle = parse_bundle(source_bytes.decode())
    source_names = package.svg_names()
    missing = sorted((source_names - EXCLUDED) - bundle.keys())
    unexpected = sorted(bundle.keys() - (source_names - EXCLUDED))
    if missing or unexpected:
        errors.append(f"Catalog name differences: missing={missing}, unexpected={unexpected}")
    if not EXCLUDED <= source_names:
        errors.append("Pinned source does not contain every explicitly recorded excluded name")
    report["catalog"] = {
        "source_svg_directory": "package/svg", "source_count": len(source_names),
        "bundled_count": len(bundle), "excluded_names": sorted(EXCLUDED),
        "excluded_count": len(EXCLUDED), "missing_names": missing, "unexpected_names": unexpected,
        "rust_source_sha256": sha256(source_bytes),
        "bundle_geometry_sha256": {name: sha256(value.encode()) for name, value in bundle.items()},
    }
    framing = {"direct_24": 0, "nested_512": 0}
    parsed = {}
    for name, geometry in bundle.items():
        try:
            root = ET.fromstring(XML_WRAPPER.format(geometry))
            tags = {local_name(element.tag) for element in root.iter()}
            if not tags <= SUPPORTED_TAGS:
                raise ValueError(f"Unexpected SVG tags: {sorted(tags - SUPPORTED_TAGS)}")
            children = list(root)
            if not children:
                raise ValueError("Empty geometry")
            if local_name(children[0].tag) == "svg":
                if len(children) != 1 or children[0].attrib != {
                    "width": "24", "height": "24", "viewBox": "0 0 512 512", "fill": "currentColor"
                }:
                    raise ValueError("Unsupported nested viewBox wrapper")
                framing["nested_512"] += 1
                parsed[name] = children[0]
            else:
                framing["direct_24"] += 1
                parsed[name] = root
        except (ET.ParseError, ValueError) as error:
            errors.append(f"{name}: {error}")
    report["framing"] = {**framing, "all_xml_well_formed": len(parsed) == len(bundle),
                         "xlink_namespace": "http://www.w3.org/1999/xlink"}
    comparisons = []
    for name in REPRESENTATIVES:
        if name not in parsed or name not in source_names:
            errors.append(f"Representative icon unavailable: {name}")
            continue
        original = ET.fromstring(package.read(f"svg/{name}.svg"))
        original_geometry = [geometry_tree(child) for child in original if local_name(child.tag) != "desc"]
        bundled_geometry = [geometry_tree(child) for child in parsed[name]]
        equal = original_geometry == bundled_geometry
        expected_box = "0 0 512 512" if name in {"about", "check", "application-screen"} else "0 0 24 24"
        if not equal or original.get("viewBox") != expected_box:
            errors.append(f"Representative geometry/framing differs: {name}")
        comparisons.append({"name": name, "source_view_box": original.get("viewBox"),
                            "normalized_geometry_equal": equal,
                            "normalized_geometry_sha256": sha256(json.dumps(original_geometry).encode())})
    report["representative_comparisons"] = comparisons
    report["normalization"] = [
        "Compare geometric child trees, retaining paths/points/groups/transforms/fill-rule/clip-rule.",
        "Ignore XML namespace spelling, attribute order and source desc metadata.",
        "Remove monochrome fill attributes to allow inherited currentColor.",
        "24-unit source outer SVG is supplied by Icon; 512-unit source retains a nested 24px SVG viewBox adapter.",
    ]
    report["errors"] = errors
    report["ok"] = not errors
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("artifact", type=Path, help="Official 3.7.0 .tgz or extracted package directory")
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--registry", type=Path, help="Optional saved official version metadata JSON")
    parser.add_argument("--report", type=Path, help="Write detailed JSON evidence here")
    args = parser.parse_args()
    try:
        report = check(args)
    except (OSError, ValueError, KeyError, IndexError, ET.ParseError, tarfile.TarError) as error:
        print(f"Icon catalog check failed: {error}", file=sys.stderr)
        return 1
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"ok": report["ok"], "source_count": report["catalog"]["source_count"],
                      "bundled_count": report["catalog"]["bundled_count"],
                      "archive_integrity_verified": report["archive_integrity_verified"],
                      "errors": report["errors"]}, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
