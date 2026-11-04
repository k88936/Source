/**
 * Flickr-style justified rows: every row shares one height, tile widths follow
 * the real aspect ratios, so nothing is cropped. Tiles come back absolutely
 * positioned - the caller only needs the block height plus one box per item.
 *
 * The last, incomplete row keeps the target height instead of being stretched
 * across the full width, which is what makes a one-image day look sane.
 */

export type Box = {
    index: number;
    left: number;
    top: number;
    width: number;
    height: number;
};

export type JustifiedLayout = {
    boxes: Box[];
    height: number;
};

/** Panorama and screenshot ratios break rows, so they are clamped. */
const MIN_RATIO = 0.42;
const MAX_RATIO = 3.6;

export function justify(
    ratios: number[],
    containerWidth: number,
    targetRowHeight: number,
    spacing = 2,
): JustifiedLayout {
    const boxes: Box[] = [];
    if (containerWidth <= 0 || targetRowHeight <= 0 || ratios.length === 0) {
        return {boxes, height: 0};
    }

    const safe = ratios.map((ratio) =>
        Number.isFinite(ratio) && ratio > 0 ? Math.min(MAX_RATIO, Math.max(MIN_RATIO, ratio)) : 1,
    );

    let rowStart = 0;
    let ratioSum = 0;
    let top = 0;

    const flush = (end: number) => {
        const count = end - rowStart;
        if (count <= 0) return;

        const available = containerWidth - spacing * (count - 1);
        const height = Math.min(targetRowHeight, available / ratioSum);
        let left = 0;

        for (let i = rowStart; i < end; i += 1) {
            const width = safe[i]! * height;
            boxes.push({index: i, left, top, width, height});
            left += width + spacing;
        }

        top += height + spacing;
        rowStart = end;
        ratioSum = 0;
    };

    for (let i = 0; i < safe.length; i += 1) {
        ratioSum += safe[i]!;
        const available = containerWidth - spacing * (i - rowStart);
        if (available / ratioSum <= targetRowHeight) {
            flush(i + 1);
        }
    }
    flush(safe.length);

    return {boxes, height: Math.max(0, top - spacing)};
}
