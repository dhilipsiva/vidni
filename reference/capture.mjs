/** Capture the live playground as 4K stills, one per narrated sentence.
 * Run through build.sh, or: PLAYWRIGHT_MODULE=… BROWSER_EXECUTABLE=… node capture.mjs
 * Reads segments.json. Writes build/stills/NNN.png, build/capture.json,
 * build/onscreen.json and build/cards/*.png. Stops if the playground shows
 * text other than the strings pinned in nibli-wasm/src/lib.rs
 * (syllogism_playground_bytes_are_verbatim), so the narration never
 * describes something that isn't on screen.
 */
import {readFile, writeFile, mkdir} from 'node:fs/promises';
import {dirname, join} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const {chromium} = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const here = dirname(fileURLToPath(import.meta.url));
const build = join(here, 'build');
const spec = JSON.parse(await readFile(join(here, 'segments.json'), 'utf8'));
const url = process.env.PLAYGROUND_URL || spec.playground;
const VIEW = {width: 1280, height: 720};
const DPR = 3;

// The pinned strings, as the playground's proof tree labels them.
const EXPECT = {
  eats: {result: 'TRUE', why: 'Because Adam is a dog, Adam is an animal; and because Adam is an animal, Adam eats.',
    rows: ['Adam eats [by recorded source #1/0: eats(every animal).]',
      'Adam is an animal [by recorded source #0/0: animal(every dog).]',
      'Adam is a dog [given]']},
  bird: {result: 'FALSE', why: 'No example could be found that satisfies the query.', rows: ['No witness found']},
  backTranslation: ['For every X, if X is a dog, then X is an animal.',
    'For every X, if X is an animal, then X eats.', 'Adam is a dog.'],
};

// Installed into the page: element lookup by key, rectangles, highlight boxes.
const HELPERS = `window.__vk = (() => {
  const BT = ${JSON.stringify(EXPECT.backTranslation)};
  const vis = el => !!el && el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden';
  const box = el => { const b = el.getBoundingClientRect(); return [b.x, b.y, b.width, b.height]; };
  const norm = s => (s || '').replace(/\\s+/g, ' ').trim();
  const clip = (el, r) => {
    let x0 = r[0], y0 = r[1], x1 = r[0] + r[2], y1 = r[1] + r[3];
    for (let p = el.parentElement; p; p = p.parentElement) {
      const cs = getComputedStyle(p);
      if (/(auto|scroll|hidden|clip)/.test(cs.overflowX + ' ' + cs.overflowY)) {
        const b = p.getBoundingClientRect();
        x0 = Math.max(x0, b.left); y0 = Math.max(y0, b.top); x1 = Math.min(x1, b.right); y1 = Math.min(y1, b.bottom);
      }
    }
    x0 = Math.max(x0, 0); y0 = Math.max(y0, 0); x1 = Math.min(x1, innerWidth); y1 = Math.min(y1, innerHeight);
    return x1 > x0 + 2 && y1 > y0 + 2 ? [x0, y0, x1 - x0, y1 - y0] : null;
  };
  const label = r => norm((r.querySelector(':scope > .proof-label') || {}).textContent);
  const rows = prefix => [...document.querySelectorAll('.proof-row')].filter(r => vis(r) && label(r).startsWith(prefix));
  const els = sel => [...document.querySelectorAll(sel)].filter(vis);
  const kbCol = () => document.querySelector('.main-row .col-tabs');
  const krLine = n => {
    const pre = document.getElementById('kb-kr-highlight');
    if (!vis(pre)) return [];
    const nodes = []; let text = '';
    const w = document.createTreeWalker(pre, NodeFilter.SHOW_TEXT);
    while (w.nextNode()) { nodes.push([w.currentNode, text.length]); text += w.currentNode.textContent; }
    const lines = text.split('\\n');
    if (n >= lines.length) return [];
    let start = 0; for (let i = 0; i < n; i++) start += lines[i].length + 1;
    const end = start + lines[n].length;
    const at = off => { for (let j = nodes.length - 1; j >= 0; j--) if (off >= nodes[j][1]) return [nodes[j][0], Math.min(off - nodes[j][1], nodes[j][0].textContent.length)]; };
    const range = document.createRange(); range.setStart(...at(start)); range.setEnd(...at(end));
    const b = range.getBoundingClientRect();
    return [{el: pre, r: [b.x - 6, b.y - 3, b.width + 12, b.height + 6]}];
  };
  const btLine = n => {
    const c = kbCol() && kbCol().querySelector('.tab-content');
    if (!c) return [];
    let el = [...c.querySelectorAll('*')].filter(e => vis(e) && norm(e.textContent) === BT[n]).pop();
    if (!el) return [];
    const cw = c.getBoundingClientRect().width;
    for (let p = el.parentElement; p && p !== c; p = p.parentElement) {
      const b = p.getBoundingClientRect();
      if (b.height > 70 || b.width > cw - 8) break;
      if (norm(p.textContent).endsWith(BT[n])) el = p;
    }
    return [{el, r: box(el)}];
  };
  const wrap = list => list.map(el => ({el, r: box(el)}));
  const KEYS = {
    'kr': () => wrap(els('.main-row .col-tabs .kr-editor')),
    'kb-status': () => wrap(els('.kb-status-summary')),
    'source-text': () => wrap(els('.main-row .col-tabs textarea')),
    'formalize': () => wrap([...document.querySelectorAll('.main-row .col-tabs button')].filter(b => vis(b) && (norm(b.textContent) === 'Formalize' || /Configure LLM/i.test(b.title || '')))),
    'examples': () => wrap(els('.app-header__examples')),
    'brand': () => wrap(els('.app-header__brand')),
    'footer': () => { const c = [...document.querySelectorAll('body *')].filter(e => vis(e) && e.textContent.includes('compiled to WebAssembly')); return c.length ? wrap([c[c.length - 1]]) : []; },
    'query-select': () => wrap(els('.query-select')),
    'query-gloss': () => wrap(els('.query-gloss')),
    'why': () => wrap(els('.proof-why')),
    'tree': () => wrap(els('.proof-tree')),
    'row:eats': () => wrap(rows('Adam eats').slice(0, 1)),
    'row:animal': () => wrap(rows('Adam is an animal').slice(0, 1)),
    'row:given': () => wrap(rows('Adam is a dog').slice(0, 1)),
    'row:detail-given': () => wrap(els('.proof-node.proof-asserted:not(.proof-leaf) > .proof-children > .proof-role-detail > .proof-row').slice(0, 1)),
    'rows:roles': () => wrap(rows('Asserted: dog')),
    'rows:proved': () => wrap(rows('(proved above)')),
    'row:failed': () => wrap(els('.proof-node.proof-failed > .proof-row')),
    'verdict': () => wrap(els('.output-entry').slice(-1)),
  };
  const resolve = key => {
    let m;
    if ((m = key.match(/^kr-line:(\\d+)$/))) return krLine(+m[1]);
    if ((m = key.match(/^bt-line:(\\d+)$/))) return btLine(+m[1]);
    if (!KEYS[key]) throw new Error('unknown highlight key ' + key);
    return KEYS[key]();
  };
  const union = rs => {
    if (!rs.length) return null;
    const x0 = Math.min(...rs.map(r => r[0])), y0 = Math.min(...rs.map(r => r[1]));
    const x1 = Math.max(...rs.map(r => r[0] + r[2])), y1 = Math.max(...rs.map(r => r[1] + r[3]));
    return [x0, y0, x1 - x0, y1 - y0];
  };
  const one = sel => { const e = document.querySelector(sel); return e && vis(e) ? box(e) : null; };
  return {
    highlight(keys) {
      const layer = document.getElementById('vk-layer');
      layer.replaceChildren();
      const out = {};
      for (const key of keys) {
        const found = resolve(key);
        if (!found.length) throw new Error('highlight key matched nothing on screen: ' + key);
        out[key] = [];
        const clipped = found.map(({el, r}) => clip(el, r)).filter(Boolean);
        // A group of rows reads as one block, so it gets one box.
        const boxes = key.startsWith('rows:') && clipped.length ? [union(clipped)] : clipped;
        for (const c of boxes) {
          out[key].push(c);
          const d = document.createElement('div');
          d.className = 'vk-box';
          Object.assign(d.style, {left: (c[0] - 4) + 'px', top: (c[1] - 4) + 'px', width: (c[2] + 8) + 'px', height: (c[3] + 8) + 'px'});
          layer.appendChild(d);
        }
        if (!out[key].length) throw new Error('highlight key is scrolled out of view: ' + key);
      }
      return out;
    },
    cameras(hl) {
      const kb = one('.main-row .col-tabs'), proof = one('.main-row .col-proof');
      const query = one('.query-row .col-tabs'), log = one('.output-log-container');
      const header = one('.app-header');
      return {
        full: [0, 0, innerWidth, innerHeight],
        app: union([header, kb, proof, query, log].filter(Boolean)),
        kb, proof, query, log,
        top: one('.main-row'),
        left: union([kb, query].filter(Boolean)),
        right: union([proof, log].filter(Boolean)),
        hl: union(Object.values(hl).flat()),
      };
    },
    scrollProof(top) { const p = document.querySelector('.col-proof .proof-panel'); p.scrollTop = top; return p.scrollTop; },
    scrollProofTo(key) {
      const p = document.querySelector('.col-proof .proof-panel');
      const found = resolve(key);
      if (!found.length) throw new Error('scroll target matched nothing: ' + key);
      p.scrollTop += found[0].el.getBoundingClientRect().top - p.getBoundingClientRect().top - 14;
      return p.scrollTop;
    },
    read() {
      const results = els('.output-result');
      return {
        result: norm(results.length ? results[results.length - 1].textContent : ''),
        why: norm((document.querySelector('.proof-why-text') || {}).textContent),
        rows: [...document.querySelectorAll('.proof-tree > .proof-node .proof-row')].filter(vis).map(label).filter(l => l && !l.startsWith('role-level')),
      };
    },
  };
})();`;

const HL_CSS = `
#vk-layer { position: fixed; inset: 0; pointer-events: none; z-index: 2147483647; }
#vk-layer .vk-box { position: absolute; box-sizing: border-box; border: 2px solid #F2542D; border-radius: 7px;
  background: rgba(242, 84, 45, 0.07); box-shadow: 0 0 0 3px rgba(242, 84, 45, 0.20), 0 0 18px rgba(242, 84, 45, 0.38); }
*, *::before, *::after { caret-color: transparent !important; }
`;

const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const fail = msg => { throw new Error(msg); };

async function tab(page, name) {
  const top = page.locator('.main-row .col-tabs');
  await top.getByRole('button', {name: name === 'nibli KR' ? /^nibli KR/ : name, exact: name !== 'nibli KR'}).click();
}

async function setGivenDetail(page, open) {
  const row = page.locator('.proof-node.proof-asserted:not(.proof-leaf) > .proof-children > .proof-role-detail > .proof-row').first();
  if (((await row.getAttribute('aria-expanded')) === 'true') !== open) await row.locator('.proof-disclosure').click();
}

async function act(page, a, onscreen) {
  if (a.tab) await tab(page, a.tab);
  else if (a.preset !== undefined) {
    const before = await page.locator('.output-entry').count();
    await page.locator('.query-select').selectOption(String(a.preset));
    await page.waitForFunction(n => document.querySelectorAll('.output-entry').length > n, before);
    await page.waitForTimeout(700);
    if (String(a.preset) === '2') onscreen.bird = await page.evaluate(() => window.__vk.read());
  } else if (a.expand === 'given-detail') await setGivenDetail(page, true);
  else if (a.collapse === 'given-detail') await setGivenDetail(page, false);
  else if (a.scroll !== undefined) await page.evaluate(t => window.__vk.scrollProof(t), a.scroll);
  else if (a.scrollTo) await page.evaluate(k => window.__vk.scrollProofTo(k), a.scrollTo);
  else fail('unknown action ' + JSON.stringify(a));
}

async function renderCards(browser) {
  const out = join(build, 'cards');
  await mkdir(out, {recursive: true});
  const jobs = [
    ['intro', '16x9', 1920, 1080, false], ['intro', '9x16', 1080, 1920, false],
    ['end', '16x9', 1920, 1080, false], ['end', '9x16', 1080, 1920, false],
    ['verdicts', '16x9', 1600, 150, true], ['verdicts', '9x16', 1000, 330, true],
    ['firewall', '16x9', 1600, 190, true], ['firewall', '9x16', 1000, 330, true],
  ];
  for (const [v, r, width, height, transparent] of jobs) {
    const page = await browser.newPage({viewport: {width, height}, deviceScaleFactor: 1});
    await page.goto(`${pathToFileURL(join(here, 'cards', 'card.html')).href}?v=${v}&r=${r}`);
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({path: join(out, `${v}-${r}.png`), omitBackground: transparent});
    await page.close();
  }
  return jobs.length;
}

const browser = await chromium.launch({headless: true, executablePath: process.env.BROWSER_EXECUTABLE});
try {
  await mkdir(join(build, 'stills'), {recursive: true});
  const page = await browser.newPage({viewport: VIEW, deviceScaleFactor: DPR, colorScheme: 'dark', reducedMotion: 'reduce'});
  const errors = [];
  page.on('pageerror', e => errors.push(String(e)));
  await page.goto(url, {waitUntil: 'networkidle'});
  await page.locator('.app-shell').waitFor({timeout: 30000});
  await page.locator('.app-header__examples').selectOption('0');
  await page.locator('.output-result').first().waitFor({timeout: 30000});
  await page.evaluate(() => document.fonts.ready);
  await page.addStyleTag({content: HL_CSS});
  await page.addScriptTag({content: HELPERS});
  await page.evaluate(() => { const d = document.createElement('div'); d.id = 'vk-layer'; document.body.appendChild(d); });

  // The syllogism must show exactly what the narration describes.
  const onscreen = {url, checkedAt: new Date().toISOString(), eats: await page.evaluate(() => window.__vk.read())};
  await tab(page, 'Back-translation');
  await page.waitForTimeout(300);
  onscreen.backTranslation = await page.evaluate(() => {
    const c = document.querySelector('.main-row .col-tabs .tab-content');
    return [...c.querySelectorAll('*')].map(e => e.textContent.replace(/\s+/g, ' ').trim())
      .filter(t => /^(For every X|Adam is a dog\.$)/.test(t)).filter((t, i, a) => a.indexOf(t) === i);
  });
  await tab(page, 'Source');
  if (!eq(onscreen.eats, EXPECT.eats)) fail('eats(Adam). differs from the pinned output:\n' + JSON.stringify(onscreen.eats, null, 1));
  if (!eq(onscreen.backTranslation, EXPECT.backTranslation)) fail('back-translation differs:\n' + JSON.stringify(onscreen.backTranslation, null, 1));

  const shots = [];
  let k = 0;
  for (const seg of spec.segments) {
    for (const s of seg.sentences) {
      const id = String(k).padStart(3, '0');
      if (s.card) {
        shots.push({k, segment: seg.id, card: s.card, overlay: s.overlay || null});
        k++;
        continue;
      }
      for (const a of s.do || []) await act(page, a, onscreen);
      await page.evaluate(() => document.activeElement && document.activeElement.blur && document.activeElement.blur());
      await page.waitForTimeout(250);
      const hl = await page.evaluate(keys => window.__vk.highlight(keys), s.hl || []);
      const cameras = await page.evaluate(h => window.__vk.cameras(h), hl);
      const still = join('stills', `${id}.png`);
      await page.screenshot({path: join(build, still), animations: 'disabled', caret: 'hide'});
      shots.push({k, segment: seg.id, still, cam: s.cam || 'app', overlay: s.overlay || null, hl, cameras});
      k++;
    }
  }
  if (!eq(onscreen.bird, EXPECT.bird)) fail('bird(Adam). differs from the pinned output:\n' + JSON.stringify(onscreen.bird, null, 1));
  if (errors.length) fail('page errors: ' + errors.join('; '));

  await writeFile(join(build, 'onscreen.json'), JSON.stringify(onscreen, null, 1));
  await writeFile(join(build, 'capture.json'), JSON.stringify({view: VIEW, dpr: DPR, shots}, null, 1));
  const cards = await renderCards(browser);
  console.log(`${shots.filter(s => s.still).length} stills, ${cards} cards; on-screen text matches the pinned syllogism output`);
} finally {
  await browser.close();
}
