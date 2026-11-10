import type {Animation} from '../types';

type Placement = {transform: string; opacity: string};

/** Where a layer sits before it slides in, and where it goes when it slides out. */
export function enterPlacement(animation: Animation): Placement {
    switch (animation) {
        case 'fade':
            return {transform: 'none', opacity: '0'};
        case 'slide_from_bottom':
            return {transform: 'translate3d(0, 100%, 0)', opacity: '1'};
        case 'none':
            return {transform: 'none', opacity: '1'};
        default:
            return {transform: 'translate3d(100%, 0, 0)', opacity: '1'};
    }
}

/** Where a layer waits while the screen above it is up. */
export function behindPlacement(animation: Animation): Placement {
    if (animation === 'slide_from_right') return {transform: 'translate3d(-30%, 0, 0)', opacity: '1'};
    return {transform: 'none', opacity: '1'};
}

export function transitionOf(animation: Animation): string {
    if (animation === 'none') return 'none';
    const ease = 'cubic-bezier(0.32, 0.72, 0, 1)';
    return `transform 280ms ${ease}, opacity 280ms ${ease}`;
}
