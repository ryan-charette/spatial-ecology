#!/usr/bin/env python3
"""Plot benchmark scaling metrics from results/benchmarks/scaling.csv."""

import csv
import os
import sys


DEFAULT_OUTPUTS = {
    "speedup": "figures/scaling_speedup.png",
    "efficiency": "figures/parallel_efficiency.png",
    "runtime": "figures/runtime_vs_patches.png",
    "overhead": "figures/event_exchange_overhead.png",
}


def main():
    input_path = sys.argv[1] if len(sys.argv) > 1 else "results/benchmarks/scaling.csv"
    output_dir = sys.argv[2] if len(sys.argv) > 2 else "figures"
    rows = load_rows(input_path)
    if not rows:
        raise SystemExit(f"no benchmark records found in {input_path}")

    outputs = {
        key: os.path.join(output_dir, os.path.basename(path))
        for key, path in DEFAULT_OUTPUTS.items()
    }
    os.makedirs(output_dir, exist_ok=True)

    try:
        plot_with_matplotlib(rows, outputs)
    except Exception:
        plot_with_pillow(rows, outputs)

    for path in outputs.values():
        print(path)


def load_rows(path):
    rows = []
    with open(path, newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            rows.append(
                {
                    "scenario": row.get("scenario", "scenario"),
                    "workers": as_float(row.get("workers")),
                    "patches": as_float(row.get("patches")),
                    "runtime": as_float(row.get("total_runtime_ms")),
                    "mean_timestep": as_float(row.get("mean_timestep_ms")),
                    "throughput": as_float(row.get("patches_per_second")),
                    "events": as_float(row.get("total_events")),
                    "cross_events": as_float(row.get("cross_worker_events")),
                    "boundary_fraction": as_float(row.get("boundary_edge_fraction")),
                    "speedup": as_float(row.get("speedup_vs_serial") or row.get("speedup")),
                    "efficiency": as_float(
                        row.get("parallel_efficiency") or row.get("efficiency")
                    ),
                }
            )
    return rows


def plot_with_matplotlib(rows, outputs):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    plot_line(
        plt,
        rows,
        "workers",
        "speedup",
        "Parallel Speedup",
        "Workers",
        "Speedup vs serial",
        outputs["speedup"],
    )
    plot_line(
        plt,
        rows,
        "workers",
        "efficiency",
        "Parallel Efficiency",
        "Workers",
        "Speedup / workers",
        outputs["efficiency"],
    )
    plot_line(
        plt,
        rows,
        "patches",
        "runtime",
        "Runtime vs Domain Size",
        "Patches",
        "Total runtime (ms)",
        outputs["runtime"],
    )
    plot_line(
        plt,
        rows,
        "workers",
        "boundary_fraction",
        "Boundary Communication Pressure",
        "Workers",
        "Boundary edge fraction",
        outputs["overhead"],
    )


def plot_line(plt, rows, x_key, y_key, title, xlabel, ylabel, output_path):
    data = sorted(
        [(row[x_key], row[y_key]) for row in rows if row[x_key] is not None and row[y_key] is not None]
    )
    if not data:
        data = [(0.0, 0.0)]
    xs, ys = zip(*data)
    fig, ax = plt.subplots(figsize=(7.5, 5.0), dpi=160)
    ax.plot(xs, ys, marker="o", linewidth=2.2, color="#2563eb")
    ax.set_title(title)
    ax.set_xlabel(xlabel)
    ax.set_ylabel(ylabel)
    ax.grid(True, color="#e5e7eb")
    fig.tight_layout()
    fig.savefig(output_path)
    plt.close(fig)


def plot_with_pillow(rows, outputs):
    draw_line_chart(
        rows,
        "workers",
        "speedup",
        "Parallel Speedup",
        "Workers",
        "Speedup vs serial",
        outputs["speedup"],
    )
    draw_line_chart(
        rows,
        "workers",
        "efficiency",
        "Parallel Efficiency",
        "Workers",
        "Speedup / workers",
        outputs["efficiency"],
    )
    draw_line_chart(
        rows,
        "patches",
        "runtime",
        "Runtime vs Domain Size",
        "Patches",
        "Total runtime (ms)",
        outputs["runtime"],
    )
    draw_line_chart(
        rows,
        "workers",
        "boundary_fraction",
        "Boundary Communication Pressure",
        "Workers",
        "Boundary edge fraction",
        outputs["overhead"],
    )


def draw_line_chart(rows, x_key, y_key, title, xlabel, ylabel, output_path):
    from PIL import Image, ImageDraw

    data = sorted(
        [(row[x_key], row[y_key]) for row in rows if row[x_key] is not None and row[y_key] is not None]
    )
    width, height = 1100, 720
    margin_left, margin_top, margin_right, margin_bottom = 130, 95, 70, 125
    plot_left, plot_top = margin_left, margin_top
    plot_right, plot_bottom = width - margin_right, height - margin_bottom
    plot_width, plot_height = plot_right - plot_left, plot_bottom - plot_top

    image = Image.new("RGB", (width, height), "white")
    draw = ImageDraw.Draw(image)
    title_font = load_font(32, bold=True)
    label_font = load_font(22)
    tick_font = load_font(17)
    small_font = load_font(15)

    draw.text((width // 2, 30), title, fill="#111827", font=title_font, anchor="ma")
    draw.text((width // 2, height - 54), xlabel, fill="#111827", font=label_font, anchor="ma")
    draw_rotated_label(image, ylabel, (40, (plot_top + plot_bottom) // 2), label_font)

    draw.rectangle((plot_left, plot_top, plot_right, plot_bottom), outline="#111827", width=2)
    if not data:
        draw.text(
            ((plot_left + plot_right) // 2, (plot_top + plot_bottom) // 2),
            "No plottable benchmark records",
            fill="#4b5563",
            font=label_font,
            anchor="mm",
        )
        image.save(output_path)
        return

    xs, ys = zip(*data)
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    if min_x == max_x:
        min_x -= 1.0
        max_x += 1.0
    else:
        x_padding = (max_x - min_x) * 0.04
        min_x -= x_padding
        max_x += x_padding
    if min_y == max_y:
        padding = max(abs(min_y) * 0.1, 1.0)
        min_y -= padding
        max_y += padding
    else:
        padding = (max_y - min_y) * 0.12
        min_y -= padding
        max_y += padding
    min_y = min(min_y, 0.0)

    for index in range(6):
        frac = index / 5
        y = plot_bottom - frac * plot_height
        value = min_y + frac * (max_y - min_y)
        draw.line((plot_left, y, plot_right, y), fill="#e5e7eb", width=1)
        draw.text((plot_left - 12, y), format_axis_value(value), fill="#374151", font=tick_font, anchor="rm")

    for value in sorted(set(xs)):
        x = project(value, min_x, max_x, plot_left, plot_right)
        draw.line((x, plot_bottom, x, plot_bottom + 6), fill="#111827", width=1)
        draw.text((x, plot_bottom + 15), format_axis_value(value), fill="#374151", font=tick_font, anchor="ma")

    points = [
        (
            project(x, min_x, max_x, plot_left, plot_right),
            project(y, min_y, max_y, plot_bottom, plot_top),
            x,
            y,
        )
        for x, y in data
    ]
    if len(points) > 1:
        draw.line([(x, y) for x, y, _, _ in points], fill="#2563eb", width=4)
    for x, y, _, value in points:
        draw.ellipse((x - 7, y - 7, x + 7, y + 7), fill="#2563eb", outline="white", width=2)
        draw.text((x, y - 18), format_axis_value(value), fill="#111827", font=small_font, anchor="ma")

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    image.save(output_path)


def project(value, source_min, source_max, target_min, target_max):
    frac = (value - source_min) / (source_max - source_min)
    return target_min + frac * (target_max - target_min)


def format_axis_value(value):
    if abs(value) >= 1000:
        return f"{value:,.0f}"
    if abs(value) >= 10:
        return f"{value:.1f}"
    return f"{value:.3f}".rstrip("0").rstrip(".")


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


def draw_rotated_label(image, text, center, font):
    from PIL import Image, ImageDraw

    scratch = Image.new("RGBA", (1, 1), (255, 255, 255, 0))
    scratch_draw = ImageDraw.Draw(scratch)
    bbox = scratch_draw.textbbox((0, 0), text, font=font)
    text_image = Image.new("RGBA", (bbox[2] - bbox[0] + 16, bbox[3] - bbox[1] + 16), (255, 255, 255, 0))
    text_draw = ImageDraw.Draw(text_image)
    text_draw.text((8, 8), text, font=font, fill="#111827")
    rotated = text_image.rotate(90, expand=True)
    x = int(center[0] - rotated.width / 2)
    y = int(center[1] - rotated.height / 2)
    image.paste(rotated, (x, y), rotated)


def as_float(value):
    if value in (None, ""):
        return None
    return float(value)


if __name__ == "__main__":
    main()
