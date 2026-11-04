#!/usr/bin/env python3
"""Headless-Chrome layout check for the PAL dashboard (spec prompt 5).

For every page, at 1920x1080 and 1280x720:
  - nothing renders off the 1920x1080 stage
  - nothing sits in the facecam corner (bottom-right 520x368)
  - subtitles stay one line, no page scrolling
Screenshots every page at both sizes and writes
results/pal/dash/layout_check.json. Exit 1 on any violation.

Usage:
  python tools/check_dash.py [--html results/pal/dash/dashboard.html]
"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

SIZES = [(1920, 1080), (1280, 720)]

BROWSER_CANDIDATES = [
    os.environ.get("CHROME_PATH", ""),
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
]


def find_browser() -> str:
    for p in BROWSER_CANDIDATES:
        if p and Path(p).exists():
            return p
    for name in ("chrome", "google-chrome", "chromium", "msedge"):
        try:
            out = subprocess.run(
                ["where" if os.name == "nt" else "which", name],
                capture_output=True, text=True, timeout=10,
            )
            if out.returncode == 0 and out.stdout.strip():
                return out.stdout.strip().splitlines()[0]
        except Exception:
            pass
    raise SystemExit("no Chrome/Edge found — set CHROME_PATH")


def check_page(browser: str, html: str, page: int, w: int, h: int):
    """Load page `page` with ?check and parse the self-report <pre id=check>."""
    url = f"file:///{html}?check=1#{page}"
    out = subprocess.run(
        [
            browser, "--headless=new", "--disable-gpu", "--no-sandbox",
            f"--window-size={w},{h}", "--virtual-time-budget=5000",
            "--dump-dom", url,
        ],
        capture_output=True, text=True, timeout=60,
    )
    m = re.search(r'<pre id="check"[^>]*>(.*?)</pre>', out.stdout, re.S)
    if not m:
        return {"error": "no self-report"}
    try:
        return json.loads(m.group(1))
    except Exception as e:
        return {"error": f"parse: {e}"}


def shot_page(browser: str, html: str, page: int, w: int, h: int, dest: Path):
    url = f"file:///{html}#{page}"
    subprocess.run(
        [
            browser, "--headless=new", "--disable-gpu", "--no-sandbox",
            f"--window-size={w},{h}", "--virtual-time-budget=4000",
            "--hide-scrollbars", f"--screenshot={dest}", url,
        ],
        capture_output=True, timeout=60,
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--html", default="results/pal/dash/dashboard.html")
    ap.add_argument("--no-shots", action="store_true")
    args = ap.parse_args()

    html = Path(args.html).resolve()
    if not html.exists():
        raise SystemExit(f"missing {html}")
    out_dir = html.parent
    shots = out_dir / "shots"
    pages = json.loads((out_dir / "pages.json").read_text())
    browser = find_browser()
    print(f"browser: {browser}\npages: {len(pages)}")

    shots.mkdir(exist_ok=True)
    report = {"sizes": {}, "ok": True}
    for w, h in SIZES:
        bad = []
        for pg in pages:
            i = pg["i"]
            r = check_page(browser, str(html).replace("\\", "/"), i, w, h)
            if "error" in r:
                bad.append({"page": i, "err": r["error"]})
            elif r.get("bad"):
                bad.append({"page": i, "bad": r["bad"]})
            if not args.no_shots:
                shot_page(
                    browser, str(html).replace("\\", "/"), i, w, h,
                    shots / f"p{i:02d}_{w}x{h}.png",
                )
        report["sizes"][f"{w}x{h}"] = bad
        if bad:
            report["ok"] = False
        print(f"{w}x{h}: {'CLEAN' if not bad else bad}")

    (out_dir / "layout_check.json").write_text(json.dumps(report, indent=1))
    print("layout_check.json written;", "PASS" if report["ok"] else "FAIL")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
