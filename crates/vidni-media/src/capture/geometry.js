window.__vidniGeometry = (() => {
  const rect = r => [r.x, r.y, r.width, r.height];
  const merge = rs => {
    const out = [];
    for (const r of rs.filter(r => r[2] > 0 && r[3] > 0).sort((a,b) => a[1]-b[1] || a[0]-b[0])) {
      const prev = out.at(-1);
      if (prev && Math.abs(prev[1]-r[1]) < 1 && Math.abs(prev[3]-r[3]) < 1 && r[0] <= prev[0]+prev[2]+1) {
        prev[2] = Math.max(prev[0]+prev[2], r[0]+r[2])-prev[0];
      } else out.push(r.slice());
    }
    return out;
  };
  const span = (el, spec) => {
    if (!spec.text || (spec.occurrence != null && (!Number.isInteger(spec.occurrence) || spec.occurrence < 1))) throw Error('text span needs nonempty text and a one-based occurrence');
    const nodes = [];
    let text = '';
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    while (walker.nextNode()) {
      nodes.push([walker.currentNode, text.length]);
      text += walker.currentNode.textContent;
    }
    const starts = [];
    for (let at=0; (at=text.indexOf(spec.text,at))>=0; at+=spec.text.length) starts.push(at);
    if (!spec.occurrence && starts.length !== 1) throw Error('missing or ambiguous text span');
    const from = starts[(spec.occurrence || 1)-1];
    if (from == null) throw Error('text span occurrence does not exist');
    const end = from+spec.text.length, boxes = [];
    for (const [node, at] of nodes) {
      const lo = Math.max(0, from-at), hi = Math.min(node.length, end-at);
      if (lo >= hi) continue;
      const range = document.createRange();
      range.setStart(node, lo); range.setEnd(node, hi);
      boxes.push(...[...range.getClientRects()].map(rect));
    }
    return merge(boxes);
  };
  return {span};
})();
