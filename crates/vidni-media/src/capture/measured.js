// Measured SVG targets: these rectangles come from the document being captured.
// No text length, font-size multiplier or handwritten text box is used.
(specs, imports) => {
  const rect = r => [r.x, r.y, r.width, r.height];
  const span = window.__vidniGeometry.span;
  const result = {};
  for (const [name, spec] of Object.entries(specs)) {
    if (!spec.element && !spec.elements?.length) continue;
    try {
      const allBoxes = [];
      for (const element of (spec.element ? [spec.element] : spec.elements)) {
      const matches = [...document.querySelectorAll('[id]')].filter(el => el.id === element);
      if (matches.length !== 1) throw Error('element ID is missing or duplicated');
      const el = matches[0], style = getComputedStyle(el);
      if (style.visibility !== 'visible' || style.display === 'none' || +style.opacity === 0) throw Error('target is hidden');
      let boxes;
      if (spec.capture) {
        const source = imports[name];
        if (el.localName !== 'image' || el.getAttribute('preserveAspectRatio') !== 'none')
          throw Error('capture target needs an SVG image with preserveAspectRatio="none"');
        const [x,y,w,h] = [el.x,el.y,el.width,el.height].map(v=>v.baseVal.value);
        const transform = el.getScreenCTM();
        boxes = source.boxes.map(([bx,by,bw,bh]) => {
          if (bx<0 || by<0 || bx+bw>source.width || by+bh>source.height) throw Error('capture target was clipped');
          const points = [[bx,by],[bx+bw,by],[bx,by+bh],[bx+bw,by+bh]].map(([px,py]) =>
            new DOMPoint(x+px*w/source.width,y+py*h/source.height).matrixTransform(transform));
          const xs=points.map(p=>p.x), ys=points.map(p=>p.y);
          return [Math.min(...xs),Math.min(...ys),Math.max(...xs)-Math.min(...xs),Math.max(...ys)-Math.min(...ys)];
        });
      } else boxes = spec.span ? span(el, spec.span) : [rect(el.getBoundingClientRect())];
      if (!boxes.length || boxes.some(b => b.some(v=>!Number.isFinite(v)) || b[2]<=0 || b[3]<=0)) throw Error('target has no visible geometry');
      for (const b of boxes) {
        if (b[0]<0 || b[1]<0 || b[0]+b[2]>innerWidth+0.01 || b[1]+b[3]>innerHeight+0.01) throw Error('target is clipped by its capture');
        for (let parent=el;parent;parent=parent.parentElement) {
          const style=getComputedStyle(parent);
          if (+style.opacity === 0 || style.display === 'none' || style.visibility !== 'visible') throw Error('target is hidden by its ancestor');
          if (style.clipPath !== 'none' || style.maskImage !== 'none') throw Error('clipped or masked targets need an unclipped measured source');
          if (parent.localName !== 'svg' || style.overflow === 'visible') continue;
          const vb=parent.viewBox.baseVal, transform=parent.getScreenCTM();
          if (!transform || !vb.width || !vb.height) continue;
          const points=[[vb.x,vb.y],[vb.x+vb.width,vb.y],[vb.x,vb.y+vb.height],[vb.x+vb.width,vb.y+vb.height]].map(([x,y])=>new DOMPoint(x,y).matrixTransform(transform));
          const xs=points.map(p=>p.x),ys=points.map(p=>p.y);
          if (b[0]<Math.min(...xs)-0.01 || b[1]<Math.min(...ys)-0.01 || b[0]+b[2]>Math.max(...xs)+0.01 || b[1]+b[3]>Math.max(...ys)+0.01)
            throw Error('target is clipped by its SVG viewport');
        }
      }
      allBoxes.push(...boxes);
      }
      result[name] = allBoxes;
    } catch (e) { throw Error(`target ${name}: ${e.message}`); }
  }
  return result;
}
