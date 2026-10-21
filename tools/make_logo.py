"""由矢量几何生成 path2obs 的 PNG / ICO 图标（紫色版）。

用法:  python tools/make_logo.py
输出:  assets/logo.png (512x512)、assets/logo.ico (多尺寸)
"""

import os

from PIL import Image, ImageDraw

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")
N = 512          # 目标画布边长
SS = 4           # 超采样倍数，用于抗锯齿
W = N * SS       # 实际绘制画布

TILE_TOP = (139, 92, 246)     # #8B5CF6
TILE_BOTTOM = (76, 29, 149)   # #4C1D95
ARROW = (109, 40, 217)        # #6D28D9

# 水晶顶点（512 坐标系）
T = (256, 96)
UL = (132, 172)
LL = (132, 340)
B = (256, 416)
LR = (380, 340)
UR = (380, 172)
C = (256, 256)

FACETS = [
    ([T, UL, LL, B, C], (255, 255, 255), (245, 243, 255)),   # 左切面
    ([T, UR, C], (255, 255, 255), (255, 255, 255)),          # 右上切面
    ([UR, LR, B, C], (237, 233, 254), (216, 204, 255)),      # 右下切面
]


def scaled(points):
    return [(x * SS, y * SS) for x, y in points]


def lerp(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(3))


def vertical_gradient(top, bottom, size=128):
    g = Image.new("RGB", (1, size))
    px = g.load()
    for y in range(size):
        px[0, y] = lerp(top, bottom, y / (size - 1))
    return g.resize((W, W), Image.BICUBIC)


def paste_shape(base, points, color_top, color_bottom, radius=None):
    mask = Image.new("L", (W, W), 0)
    d = ImageDraw.Draw(mask)
    if radius is None:
        d.polygon(scaled(points), fill=255)
    else:
        d.rounded_rectangle(
            [(points[0][0] * SS, points[0][1] * SS),
             (points[1][0] * SS, points[1][1] * SS)],
            radius=radius * SS,
            fill=255,
        )
    layer = vertical_gradient(color_top, color_bottom) if color_top != color_bottom \
        else Image.new("RGB", (W, W), color_top)
    base.paste(layer, (0, 0), mask)


def build():
    base = Image.new("RGBA", (W, W), (0, 0, 0, 0))

    # 紫色圆角底
    mask = Image.new("L", (W, W), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        [16 * SS, 16 * SS, 496 * SS, 496 * SS], radius=112 * SS, fill=255
    )
    base.paste(vertical_gradient(TILE_TOP, TILE_BOTTOM), (0, 0), mask)

    # 水晶切面
    for pts, top, bottom in FACETS:
        paste_shape(base, pts, top, bottom)

    # 箭头：杆 + 头
    paste_shape(base, [(150, 244), (322, 268)], ARROW, ARROW, radius=12)
    paste_shape(base, [(320, 220), (320, 292), (368, 256)], ARROW, ARROW)

    return base.resize((N, N), Image.LANCZOS)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    img = build()
    png = os.path.join(OUT_DIR, "logo.png")
    ico = os.path.join(OUT_DIR, "logo.ico")
    img.save(png)
    img.save(ico, sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64),
                         (128, 128), (256, 256)])
    print("wrote", os.path.normpath(png))
    print("wrote", os.path.normpath(ico))


if __name__ == "__main__":
    main()
