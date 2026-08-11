import { RangeSetBuilder } from "@codemirror/state";
import { Decoration, DecorationSet, EditorView, ViewPlugin, ViewUpdate } from "@codemirror/view";
import { expandMathShorthand } from "../shorthand/expand";
import { mathShorthandEnabled, scanMathRanges } from "../shorthand/scanner";
import { MathWidget } from "./math-widget";

function decorations(view:EditorView,globalEnabled:()=>boolean){const builder=new RangeSetBuilder<Decoration>();const source=view.state.doc.toString();if(!mathShorthandEnabled(source,globalEnabled()))return builder.finish();const selected=view.state.selection.ranges;for(const visible of view.visibleRanges){for(const range of scanMathRanges(source.slice(visible.from,visible.to))){const from=visible.from+range.from,to=visible.from+range.to;if(selected.some(s=>s.from<=to&&s.to>=from))continue;const raw=source.slice(visible.from+range.contentFrom,visible.from+range.contentTo);const expanded=expandMathShorthand(raw);if(expanded!==raw)builder.add(from,to,Decoration.replace({widget:new MathWidget(expanded,range.display),block:range.display}));}}return builder.finish();}
export function mathShorthandEditorExtension(enabled:()=>boolean){return ViewPlugin.fromClass(class {decorations:DecorationSet;constructor(view:EditorView){this.decorations=decorations(view,enabled);}update(update:ViewUpdate){if(update.docChanged||update.selectionSet||update.viewportChanged)this.decorations=decorations(update.view,enabled);}},{decorations:value=>value.decorations});}
