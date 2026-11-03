"""The sheet of borders as ground, drawn on the GPU (an offscreen OpenGL context, through moderngl).

Every spot on the sheet is one border on Mario's map: across, the way the ruler faces (a full turn, so the
right edge joins the left); up the sheet, how far it's slid. A `Ground` raises a map of true values over
some patch of the sheet (the whole of it, or a small window computed finely): flat ledges with walls for
mistakes, or one smooth surface for the squared miss. Heights are true: zero is the floor, which is drawn,
and `top` is the value that stands TALL high, the same for every picture in a chapter.

Colour is height alone, on an even ramp from gold (low) to plum ash (high); light only shapes walls and
slopes, so a flat ledge is its height's colour whichever way it faces. Round values are drawn as contours,
ledges' edges as lips, and the far ground fades into fog. A `View` is an orbit camera: it can circle,
tilt and close in, and `View.project` puts page marks exactly where the ground is drawn."""

import math
from dataclasses import dataclass, replace

import moderngl
import numpy as np

from series import style
from series.ember import rgb

WIDE, DEEP, TALL = 8.0, 4.4, 1.6  # the whole sheet in world units: across, deep, and the height of `top`
LOW, HIGH = style.WARM[2], "#4A3552"  # the ramp's ends: gold for the lowest ground, plum ash for the highest
FLOOR = "#140C0A"  # the floor at zero, under everything
FLAT = "#5A4636"  # a flat sheet's colour, before it rises
FOG = "#070505"
SUN = np.array([-0.45, -0.55, 0.70]) / np.linalg.norm([-0.45, -0.55, 0.70])
SAMPLES = 4  # multisampling, for clean edges


def _oklab(c: np.ndarray) -> np.ndarray:
    c = c / 255.0
    lin = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    m1 = np.array([[0.4122214708, 0.5363325363, 0.0514459929], [0.2119034982, 0.6806995451, 0.1073969566],
                   [0.0883024619, 0.2817188376, 0.6299787005]])
    m2 = np.array([[0.2104542553, 0.7936177850, -0.0040720468], [1.9779984951, -2.4285922050, 0.4505937099],
                   [0.0259040371, 0.7827717662, -0.8086757660]])
    return (np.cbrt(lin @ m1.T)) @ m2.T


def _rgb(lab: np.ndarray) -> np.ndarray:
    m2 = np.linalg.inv(np.array([[0.2104542553, 0.7936177850, -0.0040720468],
                                 [1.9779984951, -2.4285922050, 0.4505937099],
                                 [0.0259040371, 0.7827717662, -0.8086757660]]))
    m1 = np.linalg.inv(np.array([[0.4122214708, 0.5363325363, 0.0514459929],
                                 [0.2119034982, 0.6806995451, 0.1073969566],
                                 [0.0883024619, 0.2817188376, 0.6299787005]]))
    lin = np.clip(((lab @ m2.T) ** 3) @ m1.T, 0, 1)
    return np.where(lin <= 0.0031308, lin * 12.92, 1.055 * lin ** (1 / 2.4) - 0.055) * 255.0


def ramp(share: np.ndarray | float, low: str = LOW, high: str = HIGH) -> np.ndarray:
    """The ground's colour at a share of `top` (0 lowest, 1 highest), evenly spaced in OKLab, as RGB 0-255."""
    s = np.clip(np.asarray(share, float), 0, 1)[..., None]
    return _rgb(_oklab(rgb(low)) * (1 - s) + _oklab(rgb(high)) * s)


@dataclass(frozen=True)
class View:
    """An orbit camera: it looks at `target` (world units) from `distance` away, `pitch` radians above the
    ground and turned `yaw` radians around it (0 looks into the sheet from its near edge, the lowest offsets);
    `fov` is the vertical field of view, in degrees."""
    yaw: float = 0.0
    pitch: float = 0.9
    distance: float = 11.0
    target: tuple[float, float, float] = (0.0, 0.0, 0.3)
    fov: float = 30.0

    def eye(self) -> np.ndarray:
        cp = math.cos(self.pitch)
        return np.asarray(self.target) + self.distance * np.array(
            [-math.sin(self.yaw) * cp, -math.cos(self.yaw) * cp, math.sin(self.pitch)])

    def matrix(self, aspect: float) -> np.ndarray:
        """World to clip space, as a 4×4 (row-major; points are columns)."""
        eye, target = self.eye(), np.asarray(self.target, float)
        f = target - eye
        f /= np.linalg.norm(f)
        r = np.cross(f, [0.0, 0.0, 1.0])
        r /= np.linalg.norm(r)
        u = np.cross(r, f)
        look = np.eye(4)
        look[0, :3], look[1, :3], look[2, :3] = r, u, -f
        look[:3, 3] = -look[:3, :3] @ eye
        near, far = 0.05, 200.0
        t = 1 / math.tan(math.radians(self.fov) / 2)
        proj = np.zeros((4, 4))
        proj[0, 0], proj[1, 1] = t / aspect, t
        proj[2, 2], proj[2, 3] = (far + near) / (near - far), 2 * far * near / (near - far)
        proj[3, 2] = -1
        return proj @ look

    def project(self, world: np.ndarray, size: tuple[float, float]) -> np.ndarray:
        """World points (..., 3) to the picture, in Manim units about its centre, for a picture `size` wide
        and tall."""
        world = np.asarray(world, float)
        p = np.concatenate([world, np.ones(world.shape[:-1] + (1,))], -1) @ self.matrix(size[0] / size[1]).T
        ndc = p[..., :2] / p[..., 3:4]
        return np.stack([ndc[..., 0] * size[0] / 2, ndc[..., 1] * size[1] / 2, np.zeros(ndc.shape[:-1])], -1)

    def moved(self, **changes) -> "View":
        return replace(self, **changes)

    def between(self, other: "View", a: float) -> "View":
        """Part way to another view (yaw the short way round)."""
        turn = (other.yaw - self.yaw + math.pi) % (2 * math.pi) - math.pi
        mix = lambda p, q: p + (q - p) * a
        # distance by ratio, so closing in from far away is as smooth near the end as at the start
        far = self.distance * (other.distance / self.distance) ** a
        return View(self.yaw + turn * a, mix(self.pitch, other.pitch), far,
                    tuple(mix(np.asarray(self.target), np.asarray(other.target))), mix(self.fov, other.fov))


def world(across, up, value, top: float, lift: float = 1.0) -> np.ndarray:
    """Spots on the sheet (shares across and up, 0 to 1) at true values, in world units."""
    across, up, value = np.broadcast_arrays(np.asarray(across, float), np.asarray(up, float), np.asarray(value, float))
    return np.stack([(across - 0.5) * WIDE, (up - 0.5) * DEEP, value / top * TALL * lift], -1)


_VERTEX = """
#version 330
uniform mat4 matrix;
uniform vec4 patch;   // the ground's shares of the sheet: across from, across to, up from, up to
uniform float scale;  // world height per unit of value
uniform float base, stretch;  // heights above `base` are stretched (1: true heights)
in vec2 at;           // the ground's own shares, 0 to 1
in float h;           // this vertex's true value (its height)
in float c;           // the value its colour shows (a wall takes the ledge above it)
in vec3 n;
out vec2 v_at; out float v_h; out float v_c; out vec3 v_n; out vec3 v_world;
void main() {
    vec2 s = vec2(mix(patch.x, patch.y, at.x), mix(patch.z, patch.w, at.y));
    v_world = vec3((s.x - 0.5) * %(WIDE)f, (s.y - 0.5) * %(DEEP)f, (base + (h - base) * stretch) * scale);
    v_at = at; v_h = h; v_c = c;
    v_n = vec3(n.xy * scale * stretch, n.z);  // slopes come in value units: scale them like the heights
    gl_Position = matrix * vec4(v_world, 1.0);
}
"""

_FRAGMENT = """
#version 330
uniform sampler2D ramp_tex;
uniform usampler2D glow_tex;
uniform sampler2D value_tex;
uniform float colour_low, colour_high, flatness, contour, fog_near, fog_far, lips, drop;
uniform vec3 sun, eye, fog_colour, flat_colour, glow_colour;
uniform int wrap_across, is_floor;
uniform vec2 grid;
in vec2 v_at; in float v_h; in float v_c; in vec3 v_n; in vec3 v_world;
out vec4 colour;

float cellv(ivec2 k, ivec2 size) {
    if (wrap_across == 1) k.x = (k.x + size.x) %% size.x;
    if (k.x < 0 || k.x >= size.x || k.y < 0 || k.y >= size.y) return -1.0;
    return texelFetch(value_tex, k, 0).r;
}

void main() {
    vec3 n = normalize(v_n);
    vec3 base;
    if (is_floor == 1) {
        // the floor at zero: dark, with a faint grid (by default an eighth of a turn across, a sixth of the
        // slide up)
        vec2 g = v_at * grid;
        vec2 d = abs(fract(g - 0.5) - 0.5) / fwidth(g);
        float line = 1.0 - smoothstep(0.0, 1.2, min(d.x, d.y));
        base = mix(%(FLOOR)s, %(FLOOR)s * 2.4, line * 0.6);
    } else {
        base = texture(ramp_tex, vec2(clamp((v_c - colour_low) / (colour_high - colour_low), 0.0, 1.0), 0.5)).rgb;
        ivec2 size = textureSize(value_tex, 0);
        vec2 cell = v_at * vec2(size);
        ivec2 k = ivec2(floor(min(cell, vec2(size) - 0.001)));
        if (texelFetch(glow_tex, k, 0).r > 0u) base = mix(base, glow_colour, 0.55);
        if (lips > 0.0 && drop > 0.0 && n.z > 0.99) {
            // a ledge's edge where the ground beside it is lower by `drop` or more: a thin bright lip, about
            // lips pixels wide
            vec2 f = cell - vec2(k);
            vec2 px = fwidth(cell);
            float here = v_c, lip = 0.0;
            float l = cellv(k + ivec2(-1, 0), size), r = cellv(k + ivec2(1, 0), size);
            float b = cellv(k + ivec2(0, -1), size), t = cellv(k + ivec2(0, 1), size);
            if (l >= 0.0 && l <= here - drop) lip = max(lip, 1.0 - smoothstep(0.0, lips, f.x / px.x));
            if (r >= 0.0 && r <= here - drop) lip = max(lip, 1.0 - smoothstep(0.0, lips, (1.0 - f.x) / px.x));
            if (b >= 0.0 && b <= here - drop) lip = max(lip, 1.0 - smoothstep(0.0, lips, f.y / px.y));
            if (t >= 0.0 && t <= here - drop) lip = max(lip, 1.0 - smoothstep(0.0, lips, (1.0 - f.y) / px.y));
            base = mix(base, min(base * 1.35 + 0.06, vec3(1.0)), lip * 0.85);
        }
        // light shapes walls and slopes only: a flat top keeps its colour whichever way the light comes
        float slope = 1.0 - abs(n.z);
        float lit = 0.42 + 0.58 * max(dot(n, sun), 0.0);
        base *= mix(1.0, lit, clamp(slope * 1.6, 0.0, 1.0));
        if (contour > 0.0) {
            float g = v_h / contour;
            float d = abs(fract(g - 0.5) - 0.5) / max(fwidth(g), 1e-6);
            base = mix(base, base * 0.55, (1.0 - smoothstep(0.0, 1.1, d)) * 0.8);
        }
        base = mix(base, flat_colour, flatness);
    }
    float fog = smoothstep(fog_near, fog_far, length(v_world - eye));
    colour = vec4(mix(base, fog_colour, fog), 1.0);
}
"""


def _glsl(c: str) -> str:
    v = rgb(c) / 255.0
    return f"vec3({v[0]:.4f}, {v[1]:.4f}, {v[2]:.4f})"


_GL: dict = {}


def _context() -> tuple[moderngl.Context, moderngl.Program]:
    if not _GL:
        ctx = moderngl.create_standalone_context()
        program = ctx.program(vertex_shader=_VERTEX % {"WIDE": WIDE, "DEEP": DEEP},
                              fragment_shader=_FRAGMENT % {"FLOOR": _glsl(FLOOR)})
        table = ramp(np.linspace(0, 1, 256)).astype("u1")
        ramp_tex = ctx.texture((256, 1), 3, table.tobytes())
        ramp_tex.repeat_x = False
        _GL.update(ctx=ctx, program=program, ramp=ramp_tex)
    return _GL["ctx"], _GL["program"]


def _steps(v: np.ndarray, wrap: bool, base: float = 0.0, hole: np.ndarray | None = None) -> np.ndarray:
    """Flat tops for every cell, and a wall wherever two neighbours differ (down to `base` at the edges, and
    round a `hole` of cells left out, where a finer patch goes).
    Rows of (at.x, at.y, h, c, n.x, n.y, n.z), six per quad."""
    na, no = v.shape
    u = np.arange(na + 1) / na
    w = np.arange(no + 1) / no
    if hole is not None:
        v = np.where(hole, base, v)
    i, j = (k.ravel() for k in np.meshgrid(np.arange(na), np.arange(no), indexing="ij"))
    if hole is not None:
        i, j = i[~hole[i, j]], j[~hole[i, j]]
    h = v[i, j]
    tops = np.stack([np.stack([u[i], w[j], h, h]), np.stack([u[i + 1], w[j], h, h]),
                     np.stack([u[i + 1], w[j + 1], h, h]), np.stack([u[i], w[j + 1], h, h])]).transpose(2, 0, 1)
    quads, normals = [tops], [np.tile([0.0, 0.0, 1.0], (len(h), 1))]
    # walls across, at the boundary after column k: between it and the next (wrapping round the turn), or
    # down to the floor at the sheet's sides
    if wrap:
        a, b, at = v, np.roll(v, -1, 0), u[1:]
    else:
        pad = np.vstack([np.full((1, no), base), v, np.full((1, no), base)])
        a, b, at = pad[:-1], pad[1:], u
    ii, jj = np.nonzero(~np.isclose(a, b))
    lo, hi = np.minimum(a[ii, jj], b[ii, jj]), np.maximum(a[ii, jj], b[ii, jj])
    x = at[ii]
    quads.append(np.stack([np.stack([x, w[jj], lo, hi]), np.stack([x, w[jj + 1], lo, hi]),
                           np.stack([x, w[jj + 1], hi, hi]), np.stack([x, w[jj], hi, hi])]).transpose(2, 0, 1))
    sign = np.where(a[ii, jj] > b[ii, jj], 1.0, -1.0)  # facing the lower side
    normals.append(np.stack([sign, 0 * sign, 0 * sign], 1))
    # walls up the sheet, between offsets, and down to the floor at its near and far edges
    pad = np.hstack([np.full((na, 1), base), v, np.full((na, 1), base)])
    a, b = pad[:, :-1], pad[:, 1:]
    ii, jj = np.nonzero(~np.isclose(a, b))
    lo, hi = np.minimum(a[ii, jj], b[ii, jj]), np.maximum(a[ii, jj], b[ii, jj])
    y = w[jj]
    quads.append(np.stack([np.stack([u[ii], y, lo, hi]), np.stack([u[ii + 1], y, lo, hi]),
                           np.stack([u[ii + 1], y, hi, hi]), np.stack([u[ii], y, hi, hi])]).transpose(2, 0, 1))
    sign = np.where(a[ii, jj] > b[ii, jj], 1.0, -1.0)
    normals.append(np.stack([0 * sign, sign, 0 * sign], 1))
    corners = np.concatenate(quads)  # (quads, 4 corners, [x, y, h, c])
    n = np.repeat(np.concatenate(normals)[:, None], 6, 1)
    return np.concatenate([corners[:, [0, 1, 2, 0, 2, 3]], n], -1).reshape(-1, 7).astype("f4")


def _smooth(v: np.ndarray, wrap: bool, base: float = 0.0, span: tuple[float, float] = (WIDE, DEEP)) -> np.ndarray:
    """One surface through every cell's middle, with skirts down to `base` along its open edges. `span` is
    the patch's size in world units, for its slopes."""
    na, no = v.shape
    u = (np.arange(na) + 0.5) / na
    w = (np.arange(no) + 0.5) / no
    if wrap:  # close the turn: one more column, the first again, one turn on
        v = np.vstack([v, v[:1]])
        u = np.r_[u, 1 + u[0]]
    gx = np.gradient(v, u * span[0], axis=0)
    gy = np.gradient(v, w * span[1], axis=1)
    n = np.stack([-gx, -gy, np.ones_like(v)], -1)  # in value units; scaled in `Ground` by the height scale
    U, W = np.meshgrid(u, w, indexing="ij")
    grid = np.stack([U, W, v, v, n[..., 0], n[..., 1], n[..., 2]], -1)
    quads = [np.stack([grid[:-1, :-1], grid[1:, :-1], grid[1:, 1:], grid[:-1, 1:]], 2).reshape(-1, 4, 7)]
    edges = [(grid[:, 0], (0, -1, 0)), (grid[:, -1], (0, 1, 0))]  # near and far
    if not wrap:
        edges += [(grid[0], (-1, 0, 0)), (grid[-1], (1, 0, 0))]
    for edge, normal in edges:
        top = edge.copy()
        top[:, 4:] = normal
        floor = top.copy()
        floor[:, 2] = base
        quads.append(np.stack([floor[:-1], floor[1:], top[1:], top[:-1]], 1))
    return np.concatenate(quads)[:, [0, 1, 2, 0, 2, 3]].reshape(-1, 7).astype("f4")


class Ground:
    """True values over a patch of the sheet (`across` and `up`: its shares of the whole sheet), drawn as
    ledges or, if `smooth`, one surface. `top` is the value drawn TALL high; `contour` the step between
    contours, in true units (0 for none); `drop` the smallest drop a ledge's lip marks (0 for none). A close
    view can stretch heights above `base` by `stretch` (and say so on screen); its sides then stop at `base`.
    Colour runs from gold at `colours[0]` to plum ash at `colours[1]` (by default, zero to `top`)."""

    def __init__(self, value: np.ndarray, top: float, smooth: bool = False, across=(0.0, 1.0), up=(0.0, 1.0),
                 contour: float = 0.0, drop: float = 0.0, base: float = 0.0, stretch: float = 1.0,
                 colours: tuple[float, float] | None = None, hole: tuple[float, float, float, float] | None = None):
        self.value = np.asarray(value, float)
        self.top, self.smooth, self.across, self.up = top, smooth, tuple(across), tuple(up)
        self.contour, self.drop = contour, 0.0 if smooth else drop
        self.base, self.stretch = base, stretch
        self.colours = colours or (0.0, top)
        self.hole = hole  # shares of the sheet (across from, to, up from, to) left out, for a finer patch
        self.wrap = abs(self.across[1] - self.across[0] - 1.0) < 1e-9  # a full turn: its sides join
        self.glow = np.zeros(self.value.shape, bool)
        self._vao = None

    def __deepcopy__(self, memo) -> "Ground":
        return self  # copies (a fade's, say) share the ground and its GPU buffers

    def height(self, across: float, up: float) -> float:
        """The true value at a spot on the sheet (shares of the whole sheet), inside this patch."""
        na, no = self.value.shape
        a = (across - self.across[0]) / (self.across[1] - self.across[0])
        b = (up - self.up[0]) / (self.up[1] - self.up[0])
        return float(self.value[min(max(int(a * na), 0), na - 1), min(max(int(b * no), 0), no - 1)])

    def world(self, across, up, value=None, lift: float = 1.0) -> np.ndarray:
        """Spots on the sheet, at a value (on this ground if none), in world units, as this ground draws them."""
        if value is None:
            value = np.vectorize(self.height)(across, up)
        value = self.base + (np.asarray(value, float) - self.base) * self.stretch
        return world(across, up, value, self.top, lift)

    def _build(self, ctx, program):
        if self.smooth:
            span = ((self.across[1] - self.across[0]) * WIDE, (self.up[1] - self.up[0]) * DEEP)
            mesh = _smooth(self.value, self.wrap, self.base, span)
        else:
            mask = None
            if self.hole is not None:
                na, no = self.value.shape
                a = self.across[0] + (np.arange(na) + 0.5) / na * (self.across[1] - self.across[0])
                b = self.up[0] + (np.arange(no) + 0.5) / no * (self.up[1] - self.up[0])
                h = self.hole
                mask = ((a > h[0]) & (a < h[1]))[:, None] & ((b > h[2]) & (b < h[3]))[None, :]
            mesh = _steps(self.value, self.wrap, self.base, mask)
        self._vbo = ctx.buffer(mesh.tobytes())
        self._vao = ctx.vertex_array(program, [(self._vbo, "2f 1f 1f 3f", "at", "h", "c", "n")])
        v = self.value.T.astype("f4")  # textures are (width = across, height = up)
        self._values = ctx.texture(v.shape[::-1], 1, v.tobytes(), dtype="f4")
        self._values.filter = (moderngl.NEAREST, moderngl.NEAREST)

    def _glow_texture(self, ctx):
        g = np.ascontiguousarray(self.glow.T.astype("u1"))
        t = ctx.texture(g.shape[::-1], 1, g.tobytes(), dtype="u1", alignment=1)
        t.filter = (moderngl.NEAREST, moderngl.NEAREST)
        return t


_FLOOR_MESH = np.array([[0, 0, -0.004, 0, 0, 0, 1], [1, 0, -0.004, 0, 0, 0, 1], [1, 1, -0.004, 0, 0, 0, 1],  # just under zero
                        [0, 0, -0.004, 0, 0, 0, 1], [1, 1, -0.004, 0, 0, 0, 1], [0, 1, -0.004, 0, 0, 0, 1]], "f4")


def render(grounds: list[Ground], view: View, pixels: tuple[int, int], lift: float = 1.0, flat: float = 0.0,
           floor: bool | tuple = True, fog: tuple[float, float] = (14.0, 30.0), lips: float = 1.6) -> np.ndarray:
    """The grounds seen from `view`, as an RGBA picture `pixels` wide and tall; transparent where nothing
    is drawn. `lift` scales every height (0 lays the ground flat on the floor); `flat` (0 to 1) blends every
    colour toward one plain colour, for a sheet before it means anything. `floor` may instead be (across from,
    to, up from, to, grid lines across, grid lines up): a floor under that patch alone."""
    ctx, program = _context()
    W, H = pixels
    msaa = ctx.framebuffer(ctx.renderbuffer((W, H), samples=SAMPLES), ctx.depth_renderbuffer((W, H), samples=SAMPLES))
    out = ctx.framebuffer(ctx.renderbuffer((W, H)))
    msaa.use()
    ctx.clear(0, 0, 0, 0)
    ctx.enable(moderngl.DEPTH_TEST)
    program["matrix"].write(view.matrix(W / H).T.astype("f4").tobytes())
    program["sun"].value = tuple(SUN)
    program["eye"].value = tuple(view.eye())
    program["fog_colour"].value = tuple(rgb(FOG) / 255)
    program["flat_colour"].value = tuple(rgb(FLAT) / 255)
    program["glow_colour"].value = tuple(rgb(style.WARM[0]) / 255)
    program["fog_near"].value, program["fog_far"].value = fog
    program["flatness"].value = flat
    program["lips"].value = lips
    _GL["ramp"].use(0)
    program["ramp_tex"].value, program["glow_tex"].value, program["value_tex"].value = 0, 1, 2
    if floor:
        if "floor" not in _GL:
            _GL["floor"] = ctx.vertex_array(program, [(ctx.buffer(_FLOOR_MESH.tobytes()), "2f 1f 1f 3f", "at", "h", "c", "n")])
        if "blank" not in _GL:  # the floor reads no textures, but every sampler needs one of its kind
            _GL["blank"] = ctx.texture((1, 1), 1, b"\0", dtype="u1", alignment=1)
            _GL["blank_f"] = ctx.texture((1, 1), 1, b"\0\0\0\0", dtype="f4")
        _GL["blank"].use(1)
        _GL["blank_f"].use(2)
        program["is_floor"].value = 1
        patch = tuple(floor) if isinstance(floor, tuple) else (0.0, 1.0, 0.0, 1.0, 8.0, 6.0)
        program["patch"].value = patch[:4]
        program["grid"].value = patch[4:6]
        program["scale"].value = 1.0
        program["contour"].value = 0.0
        program["base"].value, program["stretch"].value = 0.0, 1.0
        _GL["floor"].render()
    program["is_floor"].value = 0
    for g in grounds:
        if g._vao is None:
            g._build(ctx, program)
        glow = g._glow_texture(ctx)
        glow.use(1)
        g._values.use(2)
        program["colour_low"].value, program["colour_high"].value = g.colours
        program["scale"].value = TALL / g.top * max(lift, 1e-4)
        program["patch"].value = (g.across[0], g.across[1], g.up[0], g.up[1])
        program["contour"].value = g.contour
        program["drop"].value = g.drop
        program["base"].value, program["stretch"].value = g.base, g.stretch
        program["wrap_across"].value = int(g.wrap)
        g._vao.render()
        glow.release()
    ctx.copy_framebuffer(out, msaa)
    data = np.frombuffer(out.read(components=4), "u1").reshape(H, W, 4)[::-1].copy()
    for fb in (msaa, out):
        for a in fb.color_attachments:
            a.release()
        if fb.depth_attachment is not None:
            fb.depth_attachment.release()
        fb.release()
    return data
