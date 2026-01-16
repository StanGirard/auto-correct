import{g as W,o as q}from"./storage-Lryb-ELa.js";import{a as Y}from"./language-tool-client-B4BnbzYn.js";function V(n){const t=[],e=document.createTreeWalker(n,NodeFilter.SHOW_TEXT,{acceptNode:s=>{const i=s.parentElement;if(!i)return NodeFilter.FILTER_REJECT;const l=i.tagName.toUpperCase();if(l==="SCRIPT"||l==="STYLE"||l==="NOSCRIPT"||l==="TEMPLATE")return NodeFilter.FILTER_REJECT;const a=window.getComputedStyle(i);return a.display==="none"||a.visibility==="hidden"?NodeFilter.FILTER_REJECT:NodeFilter.FILTER_ACCEPT}});let o;for(;o=e.nextNode();)t.push(o);return t}function T(n,t){const e=n.innerText||"";if(t<0||t>=e.length)return null;const o=V(n);let s=0;for(const i of o){const l=i.textContent||"",a=s,d=s+l.length;if(t>=a&&t<d)return{node:i,offset:t-a};if(s=d,i.parentElement){const c=Q(i);if(c&&X(i,c)){if(t===s)return{node:i,offset:l.length};s++}}}if(o.length>0&&t===s){const i=o[o.length-1];return{node:i,offset:i.textContent?.length||0}}return null}function Q(n){let t=n.nextSibling;for(;t;){if(t.nodeType===Node.TEXT_NODE){if((t.textContent||"").trim())return t}else if(t.nodeType===Node.ELEMENT_NODE)return t;t=t.nextSibling}const e=n.parentElement;return e&&e.nextElementSibling?e.nextElementSibling:null}function X(n,t){const e=n.parentElement,o=t.nodeType===Node.ELEMENT_NODE?t:t.parentElement;if(!e||!o)return!1;const s=i=>{const l=window.getComputedStyle(i).display;return l==="block"||l==="list-item"||l==="flex"||l==="grid"||i.tagName==="BR"||i.tagName==="P"||i.tagName==="DIV"||i.tagName==="LI"};return!!(e!==o&&(s(e)||s(o)))}function K(n,t,e){const o=T(n,t);if(!o)return console.warn("[AutoCorrect] Could not find start position for offset:",t),null;const s=T(n,t+e);if(!s){const l=T(n,t+e-1);if(!l)return console.warn("[AutoCorrect] Could not find end position for offset:",t+e),null;const a=document.createRange();return a.setStart(o.node,o.offset),a.setEnd(l.node,Math.min(l.offset+1,l.node.length)),a}const i=document.createRange();return i.setStart(o.node,o.offset),i.setEnd(s.node,s.offset),i}function H(n){const t=new Map,e=n.innerText||"";for(let o=0;o<e.length;o++){const s=T(n,o);s&&t.set(o,s)}return t}function y(n,t){return n.get(t)||null}const j=()=>typeof CSS<"u"&&"highlights"in CSS&&typeof window.Highlight<"u";class J{element;overlay=null;shadowRoot=null;resizeObserver=null;tooltip=null;currentMatches=[];ignoredMatches=new Set;callbacks=null;boundHideTooltip;useCustomHighlights=!1;highlightStyleSheet=null;constructor(t){this.element=t,this.boundHideTooltip=this.handleOutsideClick.bind(this);const e=!(t instanceof HTMLInputElement||t instanceof HTMLTextAreaElement);this.useCustomHighlights=e&&j(),this.useCustomHighlights&&(console.log("[AutoCorrect] Using CSS Custom Highlights API (modern path)"),this.setupCustomHighlightStyles())}setupCustomHighlightStyles(){this.highlightStyleSheet=new CSSStyleSheet,this.highlightStyleSheet.replaceSync(`
      ::highlight(autocorrect-spelling) {
        text-decoration: underline wavy #EF4444;
        text-decoration-skip-ink: none;
        text-underline-offset: 2px;
      }
      ::highlight(autocorrect-grammar) {
        text-decoration: underline wavy #F59E0B;
        text-decoration-skip-ink: none;
        text-underline-offset: 2px;
      }
      ::highlight(autocorrect-style) {
        text-decoration: underline wavy #3B82F6;
        text-decoration-skip-ink: none;
        text-underline-offset: 2px;
      }
    `),document.adoptedStyleSheets=[...document.adoptedStyleSheets,this.highlightStyleSheet]}init(t){this.callbacks=t,this.createOverlay(),this.setupObservers()}createOverlay(){this.overlay=document.createElement("div"),this.overlay.className="autocorrect-overlay",this.overlay.style.cssText=`
      position: absolute;
      pointer-events: none;
      overflow: visible;
      z-index: 2147483646;
    `,this.shadowRoot=this.overlay.attachShadow({mode:"open"});const t=document.createElement("style");t.textContent=`
      :host {
        all: initial;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      }

      /* Error underlines - now clickable */
      .error-highlight {
        position: absolute;
        background: transparent;
        cursor: pointer;
        pointer-events: auto;
        border-radius: 2px;
        animation: highlightFadeIn 0.2s ease-out;
      }
      @keyframes highlightFadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
      }
      .error-highlight:hover {
        background: rgba(239, 68, 68, 0.1);
      }
      .error-highlight::after {
        content: '';
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 3px;
        background: currentColor;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='8' height='4' viewBox='0 0 8 4'%3E%3Cpath d='M0 3 Q2 0 4 3 Q6 6 8 3' stroke='%23EF4444' fill='none' stroke-width='1.5'/%3E%3C/svg%3E");
        background-repeat: repeat-x;
        background-position: bottom;
        background-size: 8px 4px;
      }
      .error-spelling::after {
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='8' height='4' viewBox='0 0 8 4'%3E%3Cpath d='M0 3 Q2 0 4 3 Q6 6 8 3' stroke='%23EF4444' fill='none' stroke-width='1.5'/%3E%3C/svg%3E");
      }
      .error-grammar::after {
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='8' height='4' viewBox='0 0 8 4'%3E%3Cpath d='M0 3 Q2 0 4 3 Q6 6 8 3' stroke='%23F59E0B' fill='none' stroke-width='1.5'/%3E%3C/svg%3E");
      }
      .error-grammar:hover {
        background: rgba(245, 158, 11, 0.1);
      }

      /* Tooltip */
      .tooltip {
        position: fixed;
        background: white;
        border-radius: 12px;
        box-shadow: 0 4px 24px rgba(0, 0, 0, 0.12), 0 0 0 1px rgba(0, 0, 0, 0.05);
        padding: 0;
        min-width: 240px;
        max-width: 320px;
        z-index: 2147483647;
        animation: tooltipIn 0.15s ease-out;
        overflow: hidden;
        pointer-events: auto;
      }
      @keyframes tooltipIn {
        from {
          opacity: 0;
          transform: translateY(-4px);
        }
        to {
          opacity: 1;
          transform: translateY(0);
        }
      }

      .tooltip-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 12px;
        background: #FAFAFA;
        border-bottom: 1px solid #F0F0F0;
      }
      .tooltip-category {
        display: flex;
        align-items: center;
        gap: 8px;
        font-weight: 600;
        font-size: 13px;
        color: #374151;
      }
      .category-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
      }
      .category-dot.spelling { background: #EF4444; }
      .category-dot.grammar { background: #F59E0B; }
      .category-dot.style { background: #3B82F6; }

      .tooltip-close {
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: none;
        background: transparent;
        color: #9CA3AF;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.15s;
      }
      .tooltip-close:hover {
        background: #E5E7EB;
        color: #374151;
      }

      .tooltip-body {
        padding: 12px;
      }
      .tooltip-message {
        color: #4B5563;
        font-size: 13px;
        line-height: 1.5;
        margin-bottom: 12px;
      }

      .tooltip-suggestions {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
      }
      .suggestion-btn {
        padding: 7px 14px;
        background: #3B82F6;
        color: white;
        border: none;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s;
        pointer-events: auto;
      }
      .suggestion-btn:hover {
        background: #2563EB;
        transform: translateY(-1px);
      }
      .suggestion-btn:active {
        transform: translateY(0);
      }

      .ignore-btn {
        padding: 7px 14px;
        background: white;
        color: #6B7280;
        border: 1px solid #E5E7EB;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s;
        pointer-events: auto;
      }
      .ignore-btn:hover {
        background: #F9FAFB;
        border-color: #D1D5DB;
      }
    `,this.shadowRoot.appendChild(t),document.body.appendChild(this.overlay),this.updatePosition()}setupObservers(){this.resizeObserver=new ResizeObserver(()=>{this.updatePosition()}),this.resizeObserver.observe(this.element),this.element.addEventListener("scroll",()=>{this.updatePosition(),this.hideTooltip()}),window.addEventListener("scroll",()=>{this.updatePosition(),this.hideTooltip()},!0),window.addEventListener("resize",()=>{this.updatePosition(),this.hideTooltip()}),this.element.addEventListener("input",()=>{this.hideTooltip()})}updatePosition(){if(!this.overlay)return;const t=this.element.getBoundingClientRect(),e=window.scrollX,o=window.scrollY;this.overlay.style.left=`${t.left+e}px`,this.overlay.style.top=`${t.top+o}px`,this.overlay.style.width=`${t.width}px`,this.overlay.style.height=`${t.height}px`}render(t,e){this.currentMatches=t,this.hideTooltip();const o=t.filter(r=>{const c=`${r.offset}-${r.length}-${r.rule.id}`;return!this.ignoredMatches.has(c)});if(console.log("[AutoCorrect] Rendering",o.length,"matches (useCustomHighlights:",this.useCustomHighlights,")"),o.length===0){this.clearHighlights();return}if(this.useCustomHighlights){this.renderWithCustomHighlights(o,e);return}if(!this.shadowRoot)return;this.shadowRoot.querySelectorAll(".error-highlight").forEach(r=>r.remove());const i=this.getVisibleTextRange(e),l=o.filter(r=>{const c=r.offset+r.length,h=Math.max(0,i.startOffset-500),u=i.endOffset+500;return c>h&&r.offset<u});console.log("[AutoCorrect] Visible matches:",l.length,"range:",i);const a=this.calculatePositions(l,e);console.log("[AutoCorrect] Positions calculated:",a);let d=0;a.forEach(r=>{const c=l[r.matchIndex];if(!c)return;const h=this.currentMatches.indexOf(c);if(r.y<-50||r.y>this.element.clientHeight+50)return;const u=document.createElement("span");u.className=`error-highlight ${this.getErrorClass(c)}`,u.style.left=`${r.x}px`,u.style.top=`${r.y}px`,u.style.width=`${r.width}px`,u.style.height=`${r.height}px`,u.dataset.matchIndex=String(h),u.addEventListener("click",g=>{g.preventDefault(),g.stopPropagation(),this.showTooltip(c,g.clientX,g.clientY,r)}),this.shadowRoot.appendChild(u),d++}),console.log("[AutoCorrect] Rendered",d,"underlines (multi-rect support enabled)")}renderWithCustomHighlights(t,e){this.clearHighlights();const o=this.element,s=H(o),i=[],l=[],a=[];t.forEach(c=>{try{const h=y(s,c.offset);if(!h)return;const u=y(s,c.offset+c.length-1);if(!u)return;const g=document.createRange();g.setStart(h.node,h.offset),g.setEnd(u.node,Math.min(u.offset+1,u.node.length));const x=c.rule.category.id.toUpperCase();x.includes("TYPO")||x.includes("SPELL")?i.push(g):x.includes("GRAMMAR")?l.push(g):a.push(g)}catch(h){console.log("[AutoCorrect] CSS Highlights range error:",h)}});const d=CSS.highlights,r=window.Highlight;i.length>0&&d.set("autocorrect-spelling",new r(...i)),l.length>0&&d.set("autocorrect-grammar",new r(...l)),a.length>0&&d.set("autocorrect-style",new r(...a)),console.log("[AutoCorrect] CSS Custom Highlights rendered:",{spelling:i.length,grammar:l.length,style:a.length}),this.renderClickTargets(t,e)}renderClickTargets(t,e){if(!this.shadowRoot)return;this.shadowRoot.querySelectorAll(".click-target").forEach(i=>i.remove()),this.calculatePositions(t,e).forEach(i=>{const l=t[i.matchIndex];if(!l)return;const a=this.currentMatches.indexOf(l),d=document.createElement("span");d.className="click-target",d.style.cssText=`
        position: absolute;
        left: ${i.x}px;
        top: ${i.y}px;
        width: ${i.width}px;
        height: ${i.height}px;
        cursor: pointer;
        pointer-events: auto;
        background: transparent;
      `,d.dataset.matchIndex=String(a),d.addEventListener("click",r=>{r.preventDefault(),r.stopPropagation(),this.showTooltip(l,r.clientX,r.clientY,i)}),this.shadowRoot.appendChild(d)})}clearHighlights(){if(this.useCustomHighlights){const t=CSS.highlights;t.delete("autocorrect-spelling"),t.delete("autocorrect-grammar"),t.delete("autocorrect-style")}this.shadowRoot&&this.shadowRoot.querySelectorAll(".error-highlight, .click-target").forEach(e=>e.remove())}getVisibleTextRange(t){const e=this.element;if(e instanceof HTMLInputElement)return{startOffset:0,endOffset:t.length};if(e instanceof HTMLTextAreaElement){const o=window.getComputedStyle(e),s=parseFloat(o.lineHeight)||parseFloat(o.fontSize)*1.2,i=e.scrollTop,l=e.clientHeight,a=Math.max(0,Math.floor(i/s)-2),d=Math.ceil((i+l)/s)+2,r=t.split(`
`);let c=0,h=t.length;for(let g=0;g<a&&g<r.length;g++)c+=r[g].length+1;let u=0;for(let g=0;g<=d&&g<r.length;g++)u+=r[g].length+1;return h=Math.min(u,t.length),{startOffset:c,endOffset:h}}return{startOffset:0,endOffset:t.length}}getErrorClass(t){const e=t.rule.category.id.toUpperCase();return e.includes("TYPO")||e.includes("SPELL")?"error-spelling":(e.includes("GRAMMAR"),"error-grammar")}getCategoryInfo(t){const e=t.rule.category.id.toUpperCase();return e.includes("TYPO")||e.includes("SPELL")?{name:"Orthographe",class:"spelling"}:e.includes("GRAMMAR")?{name:"Grammaire",class:"grammar"}:{name:"Style",class:"style"}}showTooltip(t,e,o,s){if(this.hideTooltip(),!this.shadowRoot)return;const i=this.getCategoryInfo(t);this.tooltip=document.createElement("div"),this.tooltip.className="tooltip";const l=window.innerHeight,a=150,d=l-o-20;let r=o+10;d<a&&o>a&&(r=o-a-10),this.tooltip.style.left=`${Math.min(e-20,window.innerWidth-340)}px`,this.tooltip.style.top=`${r}px`,this.tooltip.innerHTML=`
      <div class="tooltip-header">
        <div class="tooltip-category">
          <span class="category-dot ${i.class}"></span>
          <span>${i.name}</span>
        </div>
        <button class="tooltip-close" aria-label="Fermer">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 1L13 13M1 13L13 1"/>
          </svg>
        </button>
      </div>
      <div class="tooltip-body">
        <p class="tooltip-message">${t.message}</p>
        <div class="tooltip-suggestions">
          ${t.replacements.slice(0,3).map(c=>`<button class="suggestion-btn" data-replacement="${this.escapeHtml(c.value)}">${this.escapeHtml(c.value)}</button>`).join("")}
          <button class="ignore-btn">Ignorer</button>
        </div>
      </div>
    `,this.tooltip.querySelector(".tooltip-close")?.addEventListener("click",()=>{this.hideTooltip()}),this.tooltip.querySelectorAll(".suggestion-btn").forEach(c=>{c.addEventListener("click",h=>{h.stopPropagation();const u=h.target.dataset.replacement||"";console.log("[AutoCorrect] Suggestion clicked:",u),this.callbacks?.onReplace(t,u),this.hideTooltip()})}),this.tooltip.querySelector(".ignore-btn")?.addEventListener("click",()=>{const c=`${t.offset}-${t.length}-${t.rule.id}`;this.ignoredMatches.add(c),this.callbacks?.onIgnore(t),this.hideTooltip(),this.render(this.currentMatches,this.getElementText())}),this.shadowRoot.appendChild(this.tooltip),setTimeout(()=>{document.addEventListener("click",this.boundHideTooltip,!1)},10)}handleOutsideClick(t){const e=t.composedPath();this.tooltip&&!e.includes(this.tooltip)&&this.hideTooltip()}hideTooltip(){this.tooltip&&(this.tooltip.remove(),this.tooltip=null,document.removeEventListener("click",this.boundHideTooltip,!1))}escapeHtml(t){const e=document.createElement("div");return e.textContent=t,e.innerHTML}getElementText(){return this.element instanceof HTMLInputElement||this.element instanceof HTMLTextAreaElement?this.element.value:this.element.innerText||""}calculatePositions(t,e){const o=this.element instanceof HTMLInputElement,s=this.element instanceof HTMLTextAreaElement;return o||s?this.calculateInputPositions(t,e):this.calculateContentEditablePositions(t,e)}calculateInputPositions(t,e){const o=this.element,s=window.getComputedStyle(o),i=document.createElement("div");i.style.cssText=`
      position: absolute;
      top: -9999px;
      left: -9999px;
      visibility: hidden;
      white-space: pre-wrap;
      word-wrap: break-word;
      overflow-wrap: break-word;
      font-family: ${s.fontFamily};
      font-size: ${s.fontSize};
      font-weight: ${s.fontWeight};
      font-style: ${s.fontStyle};
      letter-spacing: ${s.letterSpacing};
      word-spacing: ${s.wordSpacing};
      line-height: ${s.lineHeight};
      text-transform: ${s.textTransform};
      padding: ${s.padding};
      border: ${s.borderWidth} solid transparent;
      box-sizing: border-box;
      width: ${o.offsetWidth}px;
    `,document.body.appendChild(i);const l=[],a=parseFloat(s.paddingTop)||0,d=parseFloat(s.borderTopWidth)||0,r=parseFloat(s.lineHeight)||parseFloat(s.fontSize)*1.2,c=o.scrollLeft||0,h=o.scrollTop||0;return t.forEach((u,g)=>{const x=e.substring(0,u.offset),B=e.substring(u.offset,u.offset+u.length);i.innerHTML="";const M=x.split(`
`),L=M.length-1,z=M[L],R=document.createElement("span");R.textContent=z,i.appendChild(R);const E=document.createElement("span");E.textContent=B,i.appendChild(E);const k=E.getBoundingClientRect(),D=i.getBoundingClientRect(),_=k.left-D.left-c,U=k.width||E.offsetWidth||10,G=a+d+L*r-h;l.push({x:Math.max(0,_),y:G,width:Math.max(U,10),height:r,matchIndex:g})}),document.body.removeChild(i),l}calculateContentEditablePositions(t,e){const o=[],s=this.element,i=s.getBoundingClientRect(),l=H(s);return t.forEach((a,d)=>{try{const r=y(l,a.offset);if(!r){console.log("[AutoCorrect] Could not find start position for offset:",a.offset);return}const c=y(l,a.offset+a.length-1);if(!c){console.log("[AutoCorrect] Could not find end position for offset:",a.offset+a.length-1);return}const h=document.createRange();h.setStart(r.node,r.offset),h.setEnd(c.node,Math.min(c.offset+1,c.node.length));const u=h.getClientRects();Array.from(u).forEach(g=>{o.push({x:g.left-i.left+s.scrollLeft,y:g.top-i.top+s.scrollTop,width:Math.max(g.width,10),height:g.height,matchIndex:d})})}catch(r){console.log("[AutoCorrect] Position calculation error:",r)}}),o}destroy(){if(this.hideTooltip(),this.clearHighlights(),this.resizeObserver&&this.resizeObserver.disconnect(),this.overlay&&this.overlay.remove(),this.highlightStyleSheet){const t=document.adoptedStyleSheets.indexOf(this.highlightStyleSheet);t!==-1&&(document.adoptedStyleSheets=document.adoptedStyleSheets.filter((e,o)=>o!==t)),this.highlightStyleSheet=null}}}const I=400,w=new WeakMap;let b=null,p=null;function P(n){b=n,n.enabled||document.querySelectorAll(".autocorrect-overlay").forEach(t=>t.remove())}function C(n){if(n instanceof HTMLInputElement){const t=n.type.toLowerCase();return!(!["text","search","email","url","tel",""].includes(t)||n.offsetWidth<100)}if(n instanceof HTMLTextAreaElement)return!0;if(n instanceof HTMLElement&&n.isContentEditable){if(n.offsetWidth<100||n.offsetHeight<30)return!1;const t=n.getAttribute("role");return!(t&&["button","menuitem","option","tab"].includes(t))}return!1}function f(n){let t;return n instanceof HTMLInputElement||n instanceof HTMLTextAreaElement?t=n.value:t=n.innerText||"",t.normalize("NFC")}function Z(n,t=500){const e=f(n);if(e.length<=t)return{text:e,offset:0};let o=0;if(n instanceof HTMLInputElement||n instanceof HTMLTextAreaElement)o=n.selectionStart||0;else{const r=window.getSelection();if(r&&r.rangeCount>0){const c=r.getRangeAt(0),h=document.createRange();h.selectNodeContents(n),h.setEnd(c.startContainer,c.startOffset),o=h.toString().length}}const s=e.lastIndexOf(`

`,o),i=e.indexOf(`

`,o);let l=s===-1?0:s+2,a=i===-1?e.length:i;const d=Math.floor(t/2);for(a-l<t&&(l=Math.max(0,o-d),a=Math.min(e.length,o+d));l>0&&e[l-1]!==" "&&e[l-1]!==`
`;)l--;for(;a<e.length&&e[a]!==" "&&e[a]!==`
`;)a++;return{text:e.substring(l,a),offset:l}}function O(n,t,e,o){const s=f(n),i=s.substring(t,t+e);if(console.log("[AutoCorrect] setTextContent called:",{offset:t,length:e,replacement:o,elementType:n.tagName,matchedText:`"${i}"`,contextBefore:`"${s.substring(Math.max(0,t-5),t)}"`,contextAfter:`"${s.substring(t+e,t+e+5)}"`}),i.length!==e&&console.warn("[AutoCorrect] WARNING: Matched text length mismatch!",{expected:e,actual:i.length,matchedText:`"${i}"`}),n instanceof HTMLInputElement||n instanceof HTMLTextAreaElement){const l=n.value;console.log("[AutoCorrect] Input/Textarea replacement:",{textLength:l.length,beforeOffset:t});const a=l.substring(0,t),d=l.substring(t+e);n.value=a+o+d;const r=t+o.length;n.setSelectionRange(r,r),n.dispatchEvent(new Event("input",{bubbles:!0})),console.log("[AutoCorrect] Replacement done for input/textarea")}else{console.log("[AutoCorrect] Contenteditable replacement at offset:",t,"length:",e);const l=K(n,t,e);if(!l){console.warn("[AutoCorrect] Could not create range for replacement");return}const a=l.toString();if(console.log("[AutoCorrect] Range created:",{rangeText:`"${a}"`,expectedText:`"${i}"`,matches:a===i}),a!==i&&console.warn("[AutoCorrect] WARNING: Range text mismatch! Expected:",`"${i}"`,"Got:",`"${a}"`),n.classList.contains("ck-editor__editable")||n.classList.contains("ck-content")){console.log("[AutoCorrect] CKEditor detected, using paste simulation");const r=window.getSelection();r&&(r.removeAllRanges(),r.addRange(l),setTimeout(async()=>{try{r.removeAllRanges(),r.addRange(l),await navigator.clipboard.writeText(o),console.log("[AutoCorrect] Clipboard written, triggering paste");const c=new ClipboardEvent("paste",{bubbles:!0,cancelable:!0,clipboardData:new DataTransfer});c.clipboardData?.setData("text/plain",o);const h=n.dispatchEvent(c);console.log("[AutoCorrect] Paste event dispatched, handled:",h),(!h||c.defaultPrevented)&&document.execCommand("insertText",!1,o)}catch(c){console.error("[AutoCorrect] Paste simulation failed:",c),document.execCommand("insertText",!1,o)}},10))}else n.focus(),setTimeout(()=>{try{const r=window.getSelection();if(r){r.removeAllRanges(),r.addRange(l);const c=r.toString();console.log("[AutoCorrect] Selection set:",{selectedText:`"${c}"`,expectedText:`"${i}"`,matches:c===i}),document.execCommand("insertText",!1,o)?console.log("[AutoCorrect] Replacement done via execCommand"):(console.log("[AutoCorrect] execCommand failed, trying delete + insertText"),document.execCommand("delete",!1),document.execCommand("insertText",!1,o))}}catch(r){console.error("[AutoCorrect] Error during replacement:",r)}},10);n.dispatchEvent(new InputEvent("input",{bubbles:!0,inputType:"insertText",data:o}))}}function N(n,t,e){e.debounceTimer!==null&&clearTimeout(e.debounceTimer),e.debounceTimer=window.setTimeout(()=>{e.debounceTimer=null,n()},t)}async function v(n){if(!b?.enabled||!b?.apiUrl){console.log("[AutoCorrect] Disabled or no API URL");return}const t=f(n.element);if(t===n.lastText){console.log("[AutoCorrect] Text unchanged, skipping");return}if(n.lastText=t,t.trim().length<3){n.currentMatches=[],n.renderer.render([],t);return}const{text:e,offset:o}=Z(n.element,500);console.log("[AutoCorrect] Checking text:",e.substring(0,50),"... (",e.length,"of",t.length,"chars, offset:",o,")"),console.log("[AutoCorrect] Calling API...");const i=(await Y(e,b.language,b.apiUrl)).map(l=>({...l,offset:l.offset+o}));console.log("[AutoCorrect] Got",i.length,"matches"),n.currentMatches=i,n.renderer.render(i,t)}function m(n){if(w.has(n))return;console.log("[AutoCorrect] Attaching to field:",n.tagName,n.className?.substring?.(0,50));const t=new J(n),e={element:n,renderer:t,debounceTimer:null,lastText:"",currentMatches:[]};t.init({onReplace:(o,s)=>{O(n,o.offset,o.length,s)},onIgnore:o=>{}}),w.set(n,e),n.addEventListener("input",()=>{N(()=>v(e),I,e)}),n.addEventListener("focus",()=>{p=e,f(n).trim().length>=3&&N(()=>v(e),I,e)}),n.addEventListener("blur",()=>{}),document.activeElement===n&&(p=e,f(n).trim().length>=3&&setTimeout(()=>v(e),500))}function A(n){const t=w.get(n);t&&(t.debounceTimer!==null&&clearTimeout(t.debounceTimer),t.renderer.destroy(),w.delete(n))}function S(){const n=document.querySelectorAll('input[type="text"], input[type="search"], input[type="email"], input[type="url"], input[type="tel"], input:not([type])'),t=document.querySelectorAll("textarea"),e=document.querySelectorAll('[contenteditable]:not([contenteditable="false"])');console.log("[AutoCorrect] Scan found:",n.length,"inputs,",t.length,"textareas,",e.length,"contenteditables"),n.forEach(o=>{C(o)&&m(o)}),t.forEach(o=>{m(o)}),e.forEach(o=>{o instanceof HTMLElement&&m(o)})}function F(){S(),new MutationObserver(t=>{t.forEach(e=>{if(e.type==="attributes"&&e.attributeName==="contenteditable"){const o=e.target;o instanceof HTMLElement&&C(o)&&(console.log("[AutoCorrect] Contenteditable attribute changed on:",o.tagName,o.className?.substring?.(0,50)),m(o))}e.addedNodes.forEach(o=>{o instanceof Element&&(C(o)&&m(o),o.querySelectorAll('input, textarea, [contenteditable]:not([contenteditable="false"])').forEach(s=>{C(s)&&m(s)}))}),e.removedNodes.forEach(o=>{o instanceof Element&&(A(o),o.querySelectorAll("input, textarea, [contenteditable]").forEach(A))})})}).observe(document.body,{childList:!0,subtree:!0,attributes:!0,attributeFilter:["contenteditable"]}),setTimeout(S,2e3),setTimeout(S,5e3),tt()}function tt(){chrome.runtime.onMessage.addListener((n,t,e)=>{if(n.type==="GET_MATCHES"){const o={type:"MATCHES_RESPONSE",matches:p?.currentMatches||[],textLength:p?f(p.element).length:0,fieldInfo:p?{tagName:p.element.tagName.toLowerCase(),hasContent:f(p.element).trim().length>0}:null};return e(o),!0}if(n.type==="APPLY_SUGGESTION"){const o=n;if(p&&p.currentMatches[o.matchIndex]){const s=p.currentMatches[o.matchIndex];O(p.element,s.offset,s.length,o.replacement),e({type:"SUGGESTION_APPLIED",success:!0})}else e({type:"SUGGESTION_APPLIED",success:!1});return!0}return!1})}function et(){document.querySelectorAll('input, textarea, [contenteditable="true"]').forEach(A)}async function $(){const n=await W();P(n),n.enabled&&F(),q(t=>{P(t),t.enabled?F():et()})}document.readyState==="loading"?document.addEventListener("DOMContentLoaded",$):$();
