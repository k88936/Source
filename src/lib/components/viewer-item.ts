import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';

export type ViewerItem = {
    id: string;
    name: string;
    video: boolean;
    thumb: MediaSourceParams;
    full: MediaSourceParams;
};
