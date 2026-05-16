#!/usr/bin/env python3
"""Plot a final-timestep spatial prey-density map."""

import csv
import os
import sys


TEXT = "#1f2933"


def main():
    input_path = sys.argv[1] if len(sys.argv) > 1 else "results/baseline.csv"
    output_path = sys.argv[2] if len(sys.argv) > 2 else "figures/spatial_population_map.png"
    data = load_final_grid(input_path)
    if not data["grid"]:
        raise SystemExit(f"no records found in {input_path}")

    try:
        plot_with_matplotlib(data, output_path)
    except Exception:
        plot_with_pillow(data, output_path)

    print(output_path)


def load_final_grid(path):
    rows = []
    with open(path, newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            rows.append(row)

    if not rows:
        return {"grid": []}

    first_scenario = rows[0].get("scenario", "")
    first_trial = rows[0].get("trial", "0")
    filtered = [
        row
        for row in rows
        if row.get("scenario", "") == first_scenario and row.get("trial", "0") == first_trial
    ]
    final_timestep = max(int(row["timestep"]) for row in filtered)
    final_rows = [row for row in filtered if int(row["timestep"]) == final_timestep]
    max_row = max(int(row["row"]) for row in final_rows)
    max_col = max(int(row["col"]) for row in final_rows)
    grid = [[0.0 for _ in range(max_col + 1)] for _ in range(max_row + 1)]

    for row in final_rows:
        grid[int(row["row"])][int(row["col"])] = float(row["prey"])

    return {
        "grid": grid,
        "scenario": first_scenario,
        "trial": first_trial,
        "timestep": final_timestep,
        "source": path,
    }


def plot_with_matplotlib(data, output_path):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig, ax = plt.subplots(figsize=(7, 6), dpi=160)
    image = ax.imshow(data["grid"], cmap="viridis", origin="upper")
    ax.set_xlabel("Patch column")
    ax.set_ylabel("Patch row")
    ax.set_title(f"Final Prey Density, Trial {data['trial']} at Timestep {data['timestep']}")
    fig.colorbar(image, ax=ax, label="Prey abundance")
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)


def plot_with_pillow(data, output_path):
    from PIL import Image, ImageDraw

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    width, height = 980, 820
    image = Image.new("RGB", (width, height), "white")
    draw = ImageDraw.Draw(image)

    title_font = load_font(29, bold=True)
    subtitle_font = load_font(17)
    label_font = load_font(20)
    tick_font = load_font(16)

    grid = data["grid"]
    rows = len(grid)
    cols = len(grid[0])
    max_value = max(max(row) for row in grid) or 1.0
    min_value = min(min(row) for row in grid)

    draw.text(
        (width // 2, 28),
        f"Final Prey Density, Trial {data['trial']} at Timestep {data['timestep']}",
        fill=TEXT,
        font=title_font,
        anchor="ma",
    )
    draw.text(
        (width // 2, 65),
        "Each cell is one habitat patch in the 2D spatial grid.",
        fill="#52606d",
        font=subtitle_font,
        anchor="ma",
    )

    grid_left, grid_top, grid_right, grid_bottom = 125, 115, 745, 665
    cell_w = (grid_right - grid_left) / cols
    cell_h = (grid_bottom - grid_top) / rows

    for row_index, row in enumerate(grid):
        for col_index, value in enumerate(row):
            x0 = grid_left + col_index * cell_w
            x1 = grid_left + (col_index + 1) * cell_w
            y0 = grid_top + row_index * cell_h
            y1 = grid_top + (row_index + 1) * cell_h
            scaled = (value - min_value) / max(max_value - min_value, 1.0e-9)
            draw.rectangle((x0, y0, x1, y1), fill=density_color(scaled), outline="white", width=1)

    draw.rectangle((grid_left, grid_top, grid_right, grid_bottom), outline="#2e3440", width=2)

    for col in range(cols):
        x = grid_left + (col + 0.5) * cell_w
        draw.text((x, grid_bottom + 13), str(col), fill=TEXT, font=tick_font, anchor="ma")
    for row in range(rows):
        y = grid_top + (row + 0.5) * cell_h
        draw.text((grid_left - 14, y), str(row), fill=TEXT, font=tick_font, anchor="rm")

    draw.text(((grid_left + grid_right) // 2, grid_bottom + 58), "Patch column", fill=TEXT, font=label_font, anchor="ma")
    draw_rotated_label(image, "Patch row", center=(42, (grid_top + grid_bottom) // 2), font=label_font, fill=TEXT)

    draw_colorbar(draw, x=810, y=145, width=34, height=470, min_value=min_value, max_value=max_value, font=tick_font, label_font=label_font)
    image.save(output_path)


def draw_colorbar(draw, x, y, width, height, min_value, max_value, font, label_font):
    for offset in range(height):
        value = 1.0 - offset / max(height - 1, 1)
        draw.line((x, y + offset, x + width, y + offset), fill=density_color(value), width=1)
    draw.rectangle((x, y, x + width, y + height), outline="#2e3440", width=1)
    for frac in [0.0, 0.25, 0.50, 0.75, 1.0]:
        yy = y + height - frac * height
        value = min_value + frac * (max_value - min_value)
        draw.line((x + width, yy, x + width + 7, yy), fill="#2e3440", width=1)
        draw.text((x + width + 12, yy), f"{value:.0f}", fill=TEXT, font=font, anchor="lm")
    draw.text((x + width / 2, y + height + 42), "Prey", fill=TEXT, font=label_font, anchor="ma")


def density_color(value):
    value = max(0.0, min(1.0, value))
    stops = [
        (0.0, (253, 231, 37)),
        (0.35, (94, 201, 98)),
        (0.70, (33, 145, 140)),
        (1.0, (59, 82, 139)),
    ]
    for (a_value, a_color), (b_value, b_color) in zip(stops, stops[1:]):
        if a_value <= value <= b_value:
            frac = (value - a_value) / (b_value - a_value)
            return tuple(int(a + (b - a) * frac) for a, b in zip(a_color, b_color))
    return stops[-1][1]


def load_font(size, bold=False):
    from PIL import ImageFont

    names = [
        "C:/Windows/Fonts/arialbd.ttf" if bold else "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/segoeuib.ttf" if bold else "C:/Windows/Fonts/segoeui.ttf",
        "DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf",
    ]
    for name in names:
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            pass
    return ImageFont.load_default()


def draw_rotated_label(image, text, center, font, fill):
    from PIL import Image, ImageDraw

    scratch = Image.new("RGBA", (1, 1), (255, 255, 255, 0))
    scratch_draw = ImageDraw.Draw(scratch)
    bbox = scratch_draw.textbbox((0, 0), text, font=font)
    text_image = Image.new("RGBA", (bbox[2] - bbox[0] + 12, bbox[3] - bbox[1] + 12), (255, 255, 255, 0))
    text_draw = ImageDraw.Draw(text_image)
    text_draw.text((6, 6), text, font=font, fill=fill)
    rotated = text_image.rotate(90, expand=True)
    x = int(center[0] - rotated.width / 2)
    y = int(center[1] - rotated.height / 2)
    image.paste(rotated, (x, y), rotated)


if __name__ == "__main__":
    main()
