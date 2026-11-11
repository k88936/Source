/** One measured pipe, in the coordinates of the surface it was measured on. */
export interface Route {
    id: string;
    d: string;
    /// Where the pipe enters and leaves, in surface coordinates. The window of
    /// light inside it is carried by a mask, and a mask only knows about the
    /// span it was given, not about the shape of the path.
    x1: number;
    x2: number;
    y1: number;
    y2: number;
}

/// How wide the lit window inside a pipe is, in user units. Short enough to
/// stay a slug of liquid rather than a lit half of the tube.
export const GLINT_WIDTH = 34;

/// The pace of the light at 1 MB/s, in pixels of pipe crossed per second.
/// This is the single number the animation is tuned by.
const PACE_AT_PER_MPS = 128;

/// Past this throughput the light is already as fast as the eye can read,
/// so it stops accelerating instead of turning into a blur.
const MAX_SPEED_MPS = 2;

/// A trickle below this would look like a stalled pipe, so any folder that
/// is moving bytes at all moves its light at least this fast.
const MIN_SPEED_MPS = 0.125;

const BYTES_PER_MB = 1024 * 1024;

/// A cubic that leaves and enters horizontally, so several folders feeding one
/// album look like they merge rather than cross.
export function curve(id: string, x1: number, y1: number, x2: number, y2: number): Route {
    const reach = Math.max(18, (x2 - x1) * 0.55);
    return {
        id,
        d: `M ${x1} ${y1} C ${x1 + reach} ${y1}, ${x2 - reach} ${y2}, ${x2} ${y2}`,
        x1,
        x2,
        y1,
        y2,
    };
}

/// Speed is spent on pace alone: pixels of pipe crossed per second, so two
/// folders moving the same bytes fill their tube at the same rate however
/// long the tube is, and nothing at all moves when nothing is on the wire.
export function glintPace(speed: number): number {
    const mps = Math.min(MAX_SPEED_MPS, Math.max(MIN_SPEED_MPS, speed / BYTES_PER_MB));
    return PACE_AT_PER_MPS * mps;
}

/** The window of light travelling one pipe. */
export interface Glint {
    route: Route;
    /// The band the mask is cut from, in surface coordinates.
    top: number;
    height: number;
    /// How far the window has to travel to leave the pipe.
    travel: number;
    time: string;
}

/// One travelling window per pipe that is actually moving bytes. Its span and
/// its timing both come from the measured edge, which is what keeps the light
/// crossing the whole tube instead of only the part with path data.
export function glintsOf(routes: Route[], speedOf: (id: string) => number): Glint[] {
    return routes.flatMap((route) => {
        const speed = speedOf(route.id);
        if (speed <= 0) return [];
        const travel = route.x2 - route.x1 + GLINT_WIDTH;
        return [
            {
                route,
                top: Math.min(route.y1, route.y2) - 24,
                height: Math.abs(route.y2 - route.y1) + 48,
                travel,
                time: `${(travel / glintPace(speed)).toFixed(2)}s`,
            },
        ];
    });
}
