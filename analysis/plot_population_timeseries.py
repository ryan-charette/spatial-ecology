#!/usr/bin/env python3
"""Plot relative total prey, predator, and vegetation abundance over time."""

import csv
import os
import sys
from collections import defaultdict


COLORS = {
    "prey": "#2a9d8f",
    "predators": "#d95f02",
    "vegetation": "#4c78a8",
    "grid": "#d8dee9",
    "axis": "#2e3440",
    "text": "#1f2933",
}


def main():
    input_path = sys.argv[1] if len(sys.argv) > 1 else "results/baseline.csv"
    output_path = sys.argv[2] if len(sys.argv) > 2 else "figures/population_timeseries.png"
    series = load_series(input_path)
    if not series:
        raise SystemExit(f"no records found in {input_path}")

    try:
        plot_with_matplotlib(series, output_path)
    except Exception:
        plot_with_pillow(series, output_path)

    print(output_path)


def load_series(path):
    per_trial_time = defaultdict(lambda: [0.0, 0.0, 0.0])
    trials = set()
    with open(path, newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            trial_key = (row.get("scenario", ""), row.get("trial", "0"))
            trials.add(trial_key)
            key = (trial_key[0], trial_key[1], int(row["timestep"]))
            per_trial_time[key][0] += float(row["prey"])
            per_trial_time[key][1] += float(row["predators"])
            per_trial_time[key][2] += float(row["vegetation"])

    by_time = defaultdict(list)
    for (_scenario, _trial, timestep), values in per_trial_time.items():
        by_time[timestep].append(values)

    raw = []
    for timestep in sorted(by_time):
        values = by_time[timestep]
        n = len(values)
        raw.append(
            (
                timestep,
                sum(v[0] for v in values) / n,
                sum(v[1] for v in values) / n,
                sum(v[2] for v in values) / n,
            )
        )

    initial = raw[0]
    denominators = [max(initial[1], 1.0e-9), max(initial[2], 1.0e-9), max(initial[3], 1.0e-9)]
    relative = [
        (
            timestep,
            prey / denominators[0],
            predators / denominators[1],
            vegetation / denominators[2],
        )
        for timestep, prey, predators, vegetation in raw
    ]
    return {"rows": relative, "trial_count": len(trials), "source": path}


def plot_with_matplotlib(series, output_path):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    rows = series["rows"]
    timesteps = [row[0] for row in rows]

    fig, ax = plt.subplots(figsize=(11, 6.5), dpi=160)
    ax.plot(timesteps, [row[1] for row in rows], color=COLORS["prey"], linewidth=2.4, label="Prey")
    ax.plot(
        timesteps,
        [row[2] for row in rows],
        color=COLORS["predators"],
        linewidth=2.4,
        label="Predators",
    )
    ax.plot(
        timesteps,
        [row[3] for row in rows],
        color=COLORS["vegetation"],
        linewidth=2.4,
        label="Vegetation",
    )
    ax.set_title(f"Population Trajectories, Mean Across {series['trial_count']} Trial(s)")
    ax.set_xlabel("Simulation timestep")
    ax.set_ylabel("Relative abundance (timestep 0 = 1.0)")
    ax.grid(alpha=0.25)
    ax.legend(loc="upper left", frameon=False)
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)


def plot_with_pillow(series, output_path):
    from PIL import Image, ImageDraw

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    width, height = 1200, 760
    image = Image.new("RGB", (width, height), "white")
    draw = ImageDraw.Draw(image)

    title_font = load_font(30, bold=True)
    label_font = load_font(20)
    tick_font = load_font(16)
    legend_font = load_font(18)

    plot = (105, 95, 1110, 625)
    rows = series["rows"]
    max_t = max(row[0] for row in rows) or 1
    max_y = nice_upper(max(max(row[1], row[2], row[3]) for row in rows))

    title = f"Population Trajectories, Mean Across {series['trial_count']} Trial(s)"
    draw.text((width // 2, 28), title, fill=COLORS["text"], font=title_font, anchor="ma")
    draw.text(
        (width // 2, 66),
        "Relative total abundance makes prey, predators, and vegetation comparable on one axis.",
        fill="#52606d",
        font=tick_font,
        anchor="ma",
    )

    draw_axes(
        image,
        draw,
        plot,
        x_label="Simulation timestep",
        y_label="Relative abundance (timestep 0 = 1.0)",
        x_ticks=make_ticks(0.0, float(max_t), 6),
        y_ticks=make_ticks(0.0, max_y, 6),
        x_max=float(max_t),
        y_max=max_y,
        tick_font=tick_font,
        label_font=label_font,
    )

    def xy(row, index):
        x0, y0, x1, y1 = plot
        x = x0 + (row[0] / max_t) * (x1 - x0)
        y = y1 - (row[index] / max_y) * (y1 - y0)
        return int(round(x)), int(round(y))

    for index, color in [(1, COLORS["prey"]), (2, COLORS["predators"]), (3, COLORS["vegetation"])]:
        points = [xy(row, index) for row in rows]
        if len(points) > 1:
            draw.line(points, fill=color, width=4, joint="curve")

    draw_legend(
        draw,
        items=[
            ("Prey", COLORS["prey"]),
            ("Predators", COLORS["predators"]),
            ("Vegetation", COLORS["vegetation"]),
        ],
        x=118,
        y=112,
        font=legend_font,
    )

    image.save(output_path)


def draw_axes(image, draw, plot, x_label, y_label, x_ticks, y_ticks, x_max, y_max, tick_font, label_font):
    x0, y0, x1, y1 = plot
    draw.rectangle(plot, outline=COLORS["axis"], width=2)

    for tick in y_ticks:
        y = y1 - (tick / y_max) * (y1 - y0)
        draw.line((x0, y, x1, y), fill=COLORS["grid"], width=1)
        draw.text((x0 - 12, y), f"{tick:g}", fill=COLORS["text"], font=tick_font, anchor="rm")

    for tick in x_ticks:
        x = x0 + (tick / x_max) * (x1 - x0) if x_max else x0
        draw.line((x, y0, x, y1), fill=COLORS["grid"], width=1)
        draw.text((x, y1 + 12), f"{tick:g}", fill=COLORS["text"], font=tick_font, anchor="ma")

    draw.text(((x0 + x1) // 2, y1 + 58), x_label, fill=COLORS["text"], font=label_font, anchor="ma")
    draw_rotated_label(image, y_label, center=(34, (y0 + y1) // 2), font=label_font, fill=COLORS["text"])


def draw_legend(draw, items, x, y, font):
    pad_x, pad_y = 16, 12
    row_h = 28
    width = max(draw.textlength(label, font=font) for label, _ in items) + 62
    height = len(items) * row_h + 2 * pad_y
    draw.rounded_rectangle((x, y, x + width, y + height), radius=8, fill="white", outline="#cbd2d9")
    for index, (label, color) in enumerate(items):
        yy = y + pad_y + index * row_h + 13
        draw.line((x + pad_x, yy, x + pad_x + 28, yy), fill=color, width=5)
        draw.text((x + pad_x + 42, yy), label, fill=COLORS["text"], font=font, anchor="lm")


def make_ticks(start, stop, count):
    if count <= 1 or stop <= start:
        return [start]
    step = (stop - start) / (count - 1)
    return [round(start + step * i, 2) for i in range(count)]


def nice_upper(value):
    if value <= 1.0:
        return 1.0
    return round(value * 1.12 + 0.05, 1)


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
