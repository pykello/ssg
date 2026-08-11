import { finishRenderMath, MarkdownPostProcessorContext, renderMath } from "obsidian";
import { expandMathShorthand } from "../shorthand/expand";
import { mathShorthandEnabled, scanMathRanges, sectionSource } from "../shorthand/scanner";

function renderedMathNodes(el:HTMLElement):HTMLElement[]{const descendants=Array.from(el.querySelectorAll<HTMLElement>(".math, mjx-container"));const candidates=el.matches(".math, mjx-container")?[el,...descendants]:descendants;return candidates.filter(node=>!node.closest(".math-shorthand-rendered")&&!(node.matches("mjx-container")&&node.closest(".math")));}

export function processMathShorthand(el:HTMLElement,ctx:MarkdownPostProcessorContext,enabled:boolean){const section=ctx.getSectionInfo(el);if(!section||!mathShorthandEnabled(section.text,enabled))return;const source=sectionSource(section.text,section.lineStart,section.lineEnd);const ranges=scanMathRanges(source);const nodes=renderedMathNodes(el);let changed=false;for(let i=0;i<Math.min(ranges.length,nodes.length);i++){const range=ranges[i]!,node=nodes[i]!;const raw=source.slice(range.contentFrom,range.contentTo);const expanded=expandMathShorthand(raw);if(expanded===raw)continue;const rendered=renderMath(expanded,range.display);rendered.addClass("math-shorthand-rendered");node.replaceWith(rendered);changed=true;}if(changed)finishRenderMath();}
