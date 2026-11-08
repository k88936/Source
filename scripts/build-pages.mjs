import {execFileSync} from 'node:child_process';
import {cpSync, mkdirSync, rmSync, writeFileSync} from 'node:fs';

const images = [];

async function fetchBingImages() {
    for (const idx of [0, 8, 16, 24]) {
        const url = `https://www.bing.com/HPImageArchive.aspx?format=js&idx=${idx}&n=8&mkt=zh-CN`;
        const body = await (await fetch(url)).json();
        for (const image of body.images) {
            images.push({date: image.startdate, url: image.url, copyright: image.copyright});
        }
    }
    writeFileSync('src/demo/bing.json', `${JSON.stringify(images, null, 4)}\n`);
    console.log(`demo images: ${images.length}`);
}

function run(script) {
    execFileSync('pnpm', [script], {stdio: 'inherit'});
}

await fetchBingImages();

run('demo:build');
run('site:build');

// The demo lives under the site so the iframe stays same-origin, and index.html doubles as the 404 page
// so a deep link still boots the app.
rmSync('site/dist/app', {recursive: true, force: true});
mkdirSync('site/dist/app', {recursive: true});
cpSync('dist-demo', 'site/dist/app', {recursive: true});
cpSync('site/dist/app/index.html', 'site/dist/404.html');
