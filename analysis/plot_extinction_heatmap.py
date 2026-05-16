#!/usr/bin/env python3
"""Plot extinction risk by drought probability and migration rate."""

import csv
import os
import sys
from collections import defaultdict


TEXT = "#1f2933"
GRID = "#d8dee9"


def main():
    input_path = sys.argv[1] if len(sys.argv) > 1 else "results/sweep_summary.csv"
    output_path = sys.argv[2] if len(sys.argv) > 2 else "figures/extinction_risk_heatmap.png"
    metric = sys.argv[3] if len(sys.argv) > 3 else "any_extinction"
    data = load_heatmap(input_path, metric)
    if not data["xs"] or not data["ys"]:
        raise SystemExit(f"no summary records found in {input_path}")

    try:
        plot_with_matplotlib(data, output_path)
    except Exception:
        plot_with_pillow(data, output_path)

    print(output_path)


def load_heatmap(path, metric):
    grouped = defaultdict(lambda: [0, 0])
    with open(path, newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            migration = round(float(row.get("migration_rate", 0.0)), 6)
            drought = round(float(row.get("drought_probability", 0.0)), 6)
            extinct = extinction_value(row, metric)
            grouped[(migration, drought)][0] += int(extinct)
            grouped[(migration, drought)][1] += 1

    xs = sorted({key[0] for key in grouped})
    ys = sorted({key[1] for key in grouped})
    matrix = []
    counts = []
    for drought in ys:
        row_values = []
        row_counts = []
        for migration in xs:
            extinct, total = grouped.get((migration, drought), [0, 0])
            row_values.append(extinct / total if total else 0.0)
            row_counts.append(total)
        matrix.append(row_values)
        counts.append(row_counts)

    return {
        "xs": xs,
        "ys": ys,
        "matrix": matrix,
        "counts": counts,
        "metric": metric,
        "source": path,
    }


def extinction_value(row, metric):
    prey = row.get("prey_extinct", "false").strip().lower() == "true"
    predators = row.get("predator_extinct", "false").strip().lower() == "true"
    if metric == "prey_extinct":
        return prey
    if metric == "predator_extinct":
        return predators
    return prey or predators


def plot_with_matplotlib(data, output_path):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig, ax = plt.subplots(figsize=(9, 6.5), dpi=160)
    image = ax.imshow(data["matrix"], origin="lower", cmap="magma", vmin=0.0, vmax=1.0, aspect="auto")
    ax.set_xticks(range(len(data["xs"])), [f"{x:.2f}" for x in data["xs"]])
    ax.set_yticks(range(len(data["ys"])), [format_probability(y) for y in data["ys"]])
    ax.set_xlabel("Prey migration rate")
    ax.set_ylabel("Drought probability")
    ax.set_title(title_for_metric(data["metric"]))
    for y, row in enumerate(data["matrix"]):
        for x, value in enumerate(row):
            ax.text(x, y, f"{value:.0%}", ha="center", va="center", color="white" if value > 0.45 else "black")
    fig.colorbar(image, ax=ax, label="Extinction probability")
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)


def plot_with_pillow(data, output_path):
    from PIL import Image, ImageDraw

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    width, height = 1180, 820
    image = Image.new("RGB", (width, height), "white")
    draw = ImageDraw.Draw(image)

    title_font = load_font(30, bold=True)
    subtitle_font = load_font(17)
    label_font = load_font(20)
    tick_font = load_font(16)
    cell_font = load_font(18, bold=True)

    title = title_for_metric(data["metric"])
    draw.text((width // 2, 28), title, fill=TEXT, font=title_font, anchor="ma")
    draw.text(
        (width // 2, 65),
        "Each cell is the fraction of Monte Carlo trials crossing an extinction threshold.",
        fill="#52606d",
        font=subtitle_font,
        anchor="ma",
    )

    grid_left, grid_top, grid_right, grid_bottom = 170, 115, 965, 665
    n_cols, n_rows = len(data["xs"]), len(data["ys"])
    cell_w = (grid_right - grid_left) / n_cols
    cell_h = (grid_bottom - grid_top) / n_rows

    for y_index, row in enumerate(data["matrix"]):
        for x_index, value in enumerate(row):
            x0 = grid_left + x_index * cell_w
            x1 = grid_left + (x_index + 1) * cell_w
            y0 = grid_bottom - (y_index + 1) * cell_h
            y1 = grid_bottom - y_index * cell_h
            color = heat_color(value)
            draw.rectangle((x0, y0, x1, y1), fill=color, outline="white", width=2)
            text_color = "white" if value >= 0.45 else "#111827"
            draw.text(((x0 + x1) / 2, (y0 + y1) / 2), f"{value:.0%}", fill=text_color, font=cell_font, anchor="mm")

    draw.rectangle((grid_left, grid_top, grid_right, grid_bottom), outline="#2e3440", width=2)

    for index, value in enumerate(data["xs"]):
        x = grid_left + (index + 0.5) * cell_w
        draw.text((x, grid_bottom + 13), f"{value:.2f}", fill=TEXT, font=tick_font, anchor="ma")

    for index, value in enumerate(data["ys"]):
        y = grid_bottom - (index + 0.5) * cell_h
        draw.text((grid_left - 14, y), format_probability(value), fill=TEXT, font=tick_font, anchor="rm")

    draw.text(
        ((grid_left + grid_right) // 2, grid_bottom + 58),
        "Prey migration rate",
        fill=TEXT,
        font=label_font,
        anchor="ma",
    )
    draw_rotated_label(image, "Drought probability", center=(55, (grid_top + grid_bottom) // 2), font=label_font, fill=TEXT)

    draw_colorbar(draw, x=1025, y=145, width=34, height=470, font=tick_font, label_font=label_font)
    image.save(output_path)


def draw_colorbar(draw, x, y, width, height, font, label_font):
    for offset in range(height):
        value = 1.0 - offset / max(height - 1, 1)
        draw.line((x, y + offset, x + width, y + offset), fill=heat_color(value), width=1)
    draw.rectangle((x, y, x + width, y + height), outline="#2e3440", width=1)
    for value in [0.0, 0.25, 0.50, 0.75, 1.0]:
        yy = y + height - value * height
        draw.line((x + width, yy, x + width + 7, yy), fill="#2e3440", width=1)
        draw.text((x + width + 12, yy), f"{value:.0%}", fill=TEXT, font=font, anchor="lm")
    draw.text((x + width / 2, y + height + 42), "Risk", fill=TEXT, font=label_font, anchor="ma")


def heat_color(value):
    value = max(0.0, min(1.0, value))
    stops = [
        (0.0, (255, 247, 236)),
        (0.25, (254, 196, 79)),
        (0.50, (236, 112, 20)),
        (0.75, (189, 55, 84)),
        (1.0, (76, 26, 112)),
    ]
    for (a_value, a_color), (b_value, b_color) in zip(stops, stops[1:]):
        if a_value <= value <= b_value:
            frac = (value - a_value) / (b_value - a_value)
            return tuple(int(a + (b - a) * frac) for a, b in zip(a_color, b_color))
    return stops[-1][1]


def title_for_metric(metric):
    if metric == "prey_extinct":
        return "Prey Extinction Risk by Drought and Migration"
    if metric == "predator_extinct":
        return "Predator Extinction Risk by Drought and Migration"
    return "Ecosystem Extinction Risk by Drought and Migration"


def format_probability(value):
    if 0.0 < value < 0.1:
        return f"{value:.3f}".rstrip("0").rstrip(".")
    return f"{value:.2f}"


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
