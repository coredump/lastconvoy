#!/usr/bin/env python3
"""
Convert a bitmap TTF font to a PNG sprite atlas + FontDef JSON
compatible with BitmapFont::load() in src/text.rs.

Usage:
    python3 scripts/ttf_to_bitmap_font.py <ttf> <out.png> <out.json> [options]

Options:
    --size N        Force render size (default: auto-detect 8–32)
    --chars CHARS   Character set string (default: printable ASCII 32–126)
    --spacing N     Pixels between glyphs for x_advance (default: 1)
    --space-width N Width of space character (default: half of "M" width)
    --cols N        Atlas columns (default: 32)
"""

import argparse
import json
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("Error: Pillow is required. Install with: pip install Pillow", file=sys.stderr)
    sys.exit(1)


DEFAULT_CHARS = "".join(chr(c) for c in range(32, 127))


def render_glyph(font: ImageFont.FreeTypeFont, ch: str) -> Image.Image | None:
    """Render a single character to a 1-bit image (white on black)."""
    try:
        bbox = font.getbbox(ch)
    except Exception:
        return None

    if bbox is None:
        return None

    l, t, r, b = bbox
    w = r - l
    h = b - t
    if w <= 0 or h <= 0:
        return None

    # Render into a grayscale image with some padding
    pad = 2
    img = Image.new("L", (w + pad * 2, h + pad * 2), 0)
    draw = ImageDraw.Draw(img)
    draw.text((-l + pad, -t + pad), ch, font=font, fill=255)

    # Threshold at 128 to remove anti-aliasing
    img = img.point(lambda p: 255 if p >= 128 else 0)

    # Crop to tight bounding box
    bbox2 = img.getbbox()
    if bbox2 is None:
        return None
    img = img.crop(bbox2)
    return img


def score_binary_quality(img: Image.Image) -> float:
    """Score how binary (non-anti-aliased) the glyph pixels are."""
    pixels = list(img.getdata())
    if not pixels:
        return 0.0
    binary = sum(1 for p in pixels if p == 0 or p == 255)
    return binary / len(pixels)


def auto_detect_size(ttf_path: str, test_char: str = "M") -> int:
    """Pick the render size with the most binary pixels for test_char."""
    best_size = 12
    best_score = -1.0

    for size in range(8, 33):
        try:
            font = ImageFont.truetype(ttf_path, size)
        except Exception:
            continue

        try:
            bbox = font.getbbox(test_char)
        except Exception:
            continue

        if bbox is None:
            continue

        l, t, r, b = bbox
        w, h = r - l, b - t
        if w <= 0 or h <= 0:
            continue

        pad = 2
        img = Image.new("L", (w + pad * 2, h + pad * 2), 0)
        draw = ImageDraw.Draw(img)
        draw.text((-l + pad, -t + pad), test_char, font=font, fill=255)

        score = score_binary_quality(img)
        # Prefer sizes where glyph height is a nice integer and score is high
        if score > best_score:
            best_score = score
            best_size = size

    print(f"Auto-detected size: {best_size} (binary score: {best_score:.3f})")
    return best_size


def build_atlas(
    ttf_path: str,
    size: int,
    chars: str,
    spacing: int,
    space_width: int | None,
    cols: int,
) -> tuple[Image.Image, list[dict], int]:
    """
    Build the glyph atlas and return (atlas_image, glyph_list, line_height).
    glyph_list entries: {ch, x, y, w, h, x_advance}
    """
    font = ImageFont.truetype(ttf_path, size)

    # Render all non-space glyphs
    glyphs: dict[str, Image.Image] = {}
    for ch in chars:
        if ch == " ":
            continue
        img = render_glyph(font, ch)
        if img is not None:
            glyphs[ch] = img

    # Determine M width for space
    m_img = glyphs.get("M")
    m_width = m_img.width if m_img else size // 2
    sw = space_width if space_width is not None else max(1, m_width // 2)

    # line_height = max glyph height
    if glyphs:
        line_height = max(img.height for img in glyphs.values())
    else:
        line_height = size

    # Layout: pack in grid of `cols` columns
    # Each cell is (max_glyph_w + spacing) × line_height
    max_w = max((img.width for img in glyphs.values()), default=size)
    cell_w = max_w + spacing
    cell_h = line_height

    # Build ordered list of all chars (including space)
    all_chars = [ch for ch in chars if ch in glyphs or ch == " "]
    rows = (len(all_chars) + cols - 1) // cols

    atlas_w = cols * cell_w
    atlas_h = rows * cell_h

    atlas = Image.new("RGBA", (atlas_w, atlas_h), (0, 0, 0, 0))

    glyph_list = []
    for i, ch in enumerate(all_chars):
        col = i % cols
        row = i // cols
        ax = col * cell_w
        ay = row * cell_h

        if ch == " ":
            glyph_list.append({
                "ch": ch,
                "x": ax,
                "y": ay,
                "w": 0,
                "h": 0,
                "x_advance": sw,
            })
            continue

        img = glyphs[ch]
        gw, gh = img.width, img.height

        # Convert grayscale to RGBA (white pixels on transparent)
        rgba = Image.new("RGBA", (gw, gh), (0, 0, 0, 0))
        for py in range(gh):
            for px in range(gw):
                val = img.getpixel((px, py))
                if val >= 128:
                    rgba.putpixel((px, py), (255, 255, 255, 255))

        # Center vertically in cell
        y_offset = (line_height - gh) // 2
        atlas.paste(rgba, (ax, ay + y_offset), rgba)

        glyph_list.append({
            "ch": ch,
            "x": ax,
            "y": ay + y_offset,
            "w": gw,
            "h": gh,
            "x_advance": gw + spacing,
        })

    return atlas, glyph_list, line_height


def main() -> None:
    parser = argparse.ArgumentParser(description="Convert TTF to bitmap font atlas + JSON")
    parser.add_argument("ttf", help="Input TTF path")
    parser.add_argument("png", help="Output PNG atlas path")
    parser.add_argument("json_out", help="Output JSON path")
    parser.add_argument("--size", type=int, default=None, help="Render size (default: auto)")
    parser.add_argument("--chars", default=DEFAULT_CHARS, help="Character set")
    parser.add_argument("--spacing", type=int, default=1, help="Pixel spacing for x_advance")
    parser.add_argument("--space-width", type=int, default=None, help="Space char advance width")
    parser.add_argument("--cols", type=int, default=32, help="Atlas grid columns")
    args = parser.parse_args()

    ttf_path = args.ttf
    if not Path(ttf_path).exists():
        print(f"Error: TTF not found: {ttf_path}", file=sys.stderr)
        sys.exit(1)

    size = args.size if args.size is not None else auto_detect_size(ttf_path)

    print(f"Rendering at size {size}, {len(args.chars)} characters...")
    atlas, glyph_list, line_height = build_atlas(
        ttf_path, size, args.chars, args.spacing, args.space_width, args.cols
    )

    # Save PNG
    png_path = args.png
    Path(png_path).parent.mkdir(parents=True, exist_ok=True)
    atlas.save(png_path)
    print(f"Saved atlas: {png_path} ({atlas.width}×{atlas.height})")

    # Build FontDef JSON
    font_def = {
        "line_height": line_height,
        "fallback": "?",
        "glyphs": glyph_list,
    }

    json_path = args.json_out
    Path(json_path).parent.mkdir(parents=True, exist_ok=True)
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(font_def, f, ensure_ascii=False, indent=2)
    print(f"Saved JSON: {json_path} ({len(glyph_list)} glyphs, line_height={line_height})")


if __name__ == "__main__":
    main()
