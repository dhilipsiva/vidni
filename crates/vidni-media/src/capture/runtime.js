// vidni's page runtime (TODO.md, P3-VIDNI-04), installed as window.__vidni
// in vidni's own isolated world, where the page's scripts can't reach it.
// It puts vidni's style and highlight layer into the document, finds a web
// scene's targets (docs/spec.md, Targets), draws a line's highlight boxes,
// measures the targets that cameras are framed from, reads texts for
// expectations, says where an action clicks, and calms animations. Targets arrive as the project serializes
// them. The geometry is the reference's (reference/capture.mjs), so stills
// match.
window.__vidni = (() => {
  const visible = el => !!el && el.getClientRects().length > 0 && getComputedStyle(el).visibility !== 'hidden';
  const box = el => { const b = el.getBoundingClientRect(); return [b.x, b.y, b.width, b.height]; };
  const norm = s => (s || '').replace(/\s+/g, ' ').trim();

  // A box clipped to the scroll containers around it and to the viewport;
  // null when almost none of it shows.
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
  const union = rs => {
    if (!rs.length) return null;
    const x0 = Math.min(...rs.map(r => r[0])), y0 = Math.min(...rs.map(r => r[1]));
    const x1 = Math.max(...rs.map(r => r[0] + r[2])), y1 = Math.max(...rs.map(r => r[1] + r[3]));
    return [x0, y0, x1 - x0, y1 - y0];
  };

  // A regular expression as the project writes it: JavaScript's, in Unicode
  // mode, where a leading (?i), (?m) or (?s) becomes that flag.
  const regex = pattern => {
    const m = pattern.match(/^\(\?([ims]+)\)/);
    return m ? new RegExp(pattern.slice(m[0].length), [...new Set(m[1])].join('') + 'u') : new RegExp(pattern, 'u');
  };
  const matches = (rule, text) => {
    if (typeof rule === 'string') return text === norm(rule);
    if (rule.starts_with != null) return text.startsWith(rule.starts_with);
    if (rule.contains != null) return text.includes(rule.contains);
    if (rule.matches != null) return regex(rule.matches).test(text);
    throw new Error('a text rule with no test: ' + JSON.stringify(rule));
  };
  const textOf = (el, spec) => {
    const of = spec.text_of ? el.querySelector(spec.text_of) : el;
    return norm(of ? of.textContent : '');
  };

  // One matcher's visible elements, in page order.
  const elements = spec => {
    let found = [...document.querySelectorAll(spec.css)].filter(visible);
    if (spec.text != null) found = found.filter(el => matches(spec.text, textOf(el, spec)));
    if (spec.exclude != null) found = found.filter(el => !matches(spec.exclude, textOf(el, spec)));
    if (spec.attr != null) {
      const re = regex(spec.attr.matches);
      found = found.filter(el => re.test(el.getAttribute(spec.attr.name) || ''));
    }
    return found;
  };
  // A target's elements: its matcher's, or the union of its `any`, after `pick`.
  const picked = spec => {
    let found;
    if (spec.any && spec.any.length) {
      found = [...new Set(spec.any.flatMap(elements))];
      found.sort((a, b) => a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1);
    } else {
      found = elements(spec);
    }
    if (spec.pick === 'first') return found.slice(0, 1);
    if (spec.pick === 'last') return found.slice(-1);
    return found;
  };

  // One line of an element's text, as the box of its characters, padded.
  const lineBox = (el, n) => {
    const nodes = [];
    let text = '';
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    while (walker.nextNode()) { nodes.push([walker.currentNode, text.length]); text += walker.currentNode.textContent; }
    const lines = text.split('\n');
    if (n >= lines.length) return null;
    let start = 0;
    for (let i = 0; i < n; i++) start += lines[i].length + 1;
    const end = start + lines[n].length;
    const at = off => {
      for (let j = nodes.length - 1; j >= 0; j--) {
        if (off >= nodes[j][1]) return [nodes[j][0], Math.min(off - nodes[j][1], nodes[j][0].textContent.length)];
      }
    };
    // An element with no text has no line to box.
    const from = at(start), to = at(end);
    if (!from || !to) return null;
    const range = document.createRange();
    range.setStart(...from);
    range.setEnd(...to);
    const b = range.getBoundingClientRect();
    return [b.x - 6, b.y - 3, b.width + 12, b.height + 6];
  };
  // The largest enclosing block of a match inside `within` that is at most
  // max_height tall, at least margin narrower than `within`, and whose text
  // still ends with the match's.
  const grown = (el, grow) => {
    const within = el.parentElement && el.parentElement.closest(grow.within);
    if (!within) return el;
    const text = norm(el.textContent);
    const width = within.getBoundingClientRect().width;
    for (let p = el.parentElement; p && p !== within; p = p.parentElement) {
      const b = p.getBoundingClientRect();
      if (b.height > grow.max_height || b.width > width - grow.margin) break;
      if (norm(p.textContent).endsWith(text)) el = p;
    }
    return el;
  };
  // A target's matches, each with its element and its box before clipping.
  const resolve = spec => picked(spec).flatMap(el => {
    if (spec.span != null) return window.__vidniGeometry.span(el, spec.span).map(r => ({el, r}));
    if (spec.line != null) return [{el, r: lineBox(el, spec.line)}];
    if (spec.grow != null) { const g = grown(el, spec.grow); return [{el: g, r: box(g)}]; }
    return [{el, r: box(el)}];
  }).filter(m => m.r);
  const first = (name, spec) => {
    const found = resolve(spec);
    if (!found.length) throw new Error('target ' + name + ' matches nothing on screen');
    return found[0].el;
  };
  // Where a click on an element lands. As Playwright does before a click,
  // an element not wholly showing is first scrolled into view through its
  // scroll containers, at once and centred; its box must then hold still
  // for two frames; and the click must land on it, not on something
  // covering it.
  const center = async (name, el) => {
    const shown = clip(el, box(el));
    const r0 = box(el);
    if (!shown || shown[2] < r0[2] - 1 || shown[3] < r0[3] - 1) {
      el.scrollIntoView({block: 'center', inline: 'center', behavior: 'instant'});
    }
    let last = null;
    for (let i = 0; i < 120; i++) {
      await new Promise(done => requestAnimationFrame(done));
      const now = box(el).join();
      if (now === last) break;
      last = now;
    }
    const r = el.getBoundingClientRect();
    const x = r.x + r.width / 2, y = r.y + r.height / 2;
    const hit = document.elementFromPoint(x, y);
    if (!hit || !(hit === el || el.contains(hit))) {
      throw new Error('target ' + name + ' is covered where a click would land');
    }
    return [x, y];
  };

  // vidni's style sheet, and whether its style and highlight layer are in
  // the document. They go in as soon as the document has a root and a body,
  // and go back in if the page drops them.
  let sheet = '';
  const place = () => {
    const root = document.documentElement;
    if (!root) return false;
    if (!document.getElementById('vidni-style')) {
      const s = document.createElement('style');
      s.id = 'vidni-style';
      s.textContent = sheet;
      (document.head || root).appendChild(s);
    }
    if (!document.body) return false;
    if (!document.getElementById('vidni-layer')) {
      const d = document.createElement('div');
      d.id = 'vidni-layer';
      document.body.appendChild(d);
    }
    return true;
  };

  return {
    // Timed emphasis may never silently accept a clipped or ambiguous match.
    measured(spec) {
      const els = picked(spec);
      if (!els.length || (els.length !== 1 && spec.pick !== 'all' && !spec.any?.length))
        throw Error('timed target is missing or ambiguous; select one match explicitly');
      return resolve(spec).map(({el,r}) => {
        for (let p=el;p;p=p.parentElement) {
          const cs=getComputedStyle(p);
          if (+cs.opacity === 0 || cs.display === 'none' || cs.visibility !== 'visible')
            throw Error('timed target is hidden');
          if (cs.clipPath !== 'none' || cs.maskImage !== 'none')
            throw Error('clipped or masked timed targets need an unclipped measured source');
        }
        if (spec.span) {
          const cs=getComputedStyle(el), b=el.getBoundingClientRect();
          if ((/(auto|scroll|hidden|clip)/.test(cs.overflowX) && (r[0]<b.left || r[0]+r[2]>b.right)) ||
              (/(auto|scroll|hidden|clip)/.test(cs.overflowY) && (r[1]<b.top || r[1]+r[3]>b.bottom)))
            throw Error('timed text span is clipped by its own element');
        }
        const shown=clip(el,r);
        if (!shown || shown.some((v,i)=>Math.abs(v-r[i])>0.01))
          throw Error('timed target is clipped by its capture');
        return r;
      });
    },
    // Puts vidni's style and highlight layer into the document, now or as
    // soon as it can hold them. Run at the document's start, they are there
    // from its first paint: added later, they changed how Chromium painted
    // what was under them, from one launch to the next.
    mount(css) {
      sheet = css;
      if (!place()) {
        const watch = new MutationObserver(() => { if (place()) watch.disconnect(); });
        watch.observe(document, {childList: true, subtree: true});
      }
      return true;
    },
    // Finishes the page's running animations and stops its endless ones, as
    // Playwright does for a screenshot, so that the page can settle. One the
    // page holds still (paused, or at a playback rate of 0) is left as it is.
    calm() {
      for (const a of document.getAnimations()) {
        if (!a.effect || a.playbackRate === 0 || a.playState === 'paused') continue;
        try {
          if (Number.isFinite(a.effect.getComputedTiming().endTime)) a.finish(); else a.cancel();
        } catch (e) { /* an animation that can't be finished is left running */ }
      }
      return true;
    },
    // Whether a target matches anything on screen yet.
    present(spec) { return resolve(spec).length > 0; },
    // Where clicking a target lands.
    point(name, spec) { return center(name, first(name, spec)); },
    // Where to click to open (or close) a target, or null when it already is.
    async toggle(name, spec, control, open) {
      const el = first(name, spec);
      if ((el.getAttribute('aria-expanded') === 'true') === open) return null;
      const hit = control ? el.querySelector(control) : el;
      if (!hit) throw new Error('target ' + name + ' has no ' + control + ' to click');
      return center(name, hit);
    },
    // Chooses `value` in a <select>, by value or else by label, as a person would.
    select(name, spec, value) {
      const el = first(name, spec);
      if (el.tagName !== 'SELECT') throw new Error('target ' + name + ' is not a <select>');
      const options = [...el.options];
      const option = options.find(o => o.value === value) || options.find(o => norm(o.label) === value);
      if (!option) throw new Error('target ' + name + ' has no option ' + value);
      for (const o of options) o.selected = o === option;
      el.dispatchEvent(new Event('input', {bubbles: true, composed: true}));
      el.dispatchEvent(new Event('change', {bubbles: true}));
      return true;
    },
    // Scrolls happen at once, whatever the page's scroll-behavior says.
    scroll(name, spec, top) { first(name, spec).scrollTo({top, behavior: 'instant'}); return true; },
    scrollTo(name, spec, withinName, within, offset) {
      const el = first(name, spec), box = first(withinName, within);
      const top = box.scrollTop + el.getBoundingClientRect().top - box.getBoundingClientRect().top - offset;
      box.scrollTo({top, behavior: 'instant'});
      return true;
    },
    // Draws the boxes of `highlights` ([name, spec] pairs), and returns them by name.
    highlight(highlights) {
      if (!place()) throw new Error('the page has no body to draw highlights on');
      const layer = document.getElementById('vidni-layer');
      layer.replaceChildren();
      const out = {};
      for (const [name, spec] of highlights) {
        const found = resolve(spec);
        if (!found.length) throw new Error('highlight ' + name + ' matches nothing on screen');
        const clipped = found.map(({el, r}) => clip(el, r)).filter(Boolean);
        const boxes = spec.merge && clipped.length ? [union(clipped)] : clipped;
        if (!boxes.length) throw new Error('highlight ' + name + ' is scrolled out of view');
        out[name] = boxes;
        for (const c of boxes) {
          const d = document.createElement('div');
          d.className = 'vidni-box';
          Object.assign(d.style, {left: (c[0] - 4) + 'px', top: (c[1] - 4) + 'px', width: (c[2] + 8) + 'px', height: (c[3] + 8) + 'px'});
          layer.appendChild(d);
        }
      }
      return out;
    },
    // The boxes of each of `targets` ([name, spec] pairs) as they show now,
    // by name. Cameras are framed from them when a cut is built.
    targets(targets) {
      const out = {};
      for (const [name, spec] of targets) {
        out[name] = resolve(spec).map(({el, r}) => clip(el, r)).filter(Boolean);
      }
      return out;
    },
    // The texts of a target's matches, in page order.
    read(spec) {
      const texts = picked(spec).map(el => textOf(el, spec)).filter(Boolean);
      return spec.unique ? [...new Set(texts)] : texts;
    },
  };
})();
