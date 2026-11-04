import type {MediaSourceParams} from './bindings/MediaSourceParams';

import {api} from './client';

/** The single way to address a media file. The backend picks the transport -
 * the `media` scheme on desktop, the alias Android intercepts, a signed server
 * URL where the platform cannot play a custom scheme - so no component ever
 * builds one of these addresses itself. */
export function mediaSource(params: MediaSourceParams): Promise<string> {
    return api.mediaSource(params);
}
