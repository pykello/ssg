import { EditorState, Prec, RangeSetBuilder, StateField } from "@codemirror/state";
import { Decoration, DecorationSet, EditorView } from "@codemirror/view";
import { expandMathShorthand } from "../shorthand/expand";
import { mathShorthandEnabled, scanMathRanges } from "../shorthand/scanner";
import { MathWidget } from "./math-widget";

function buildDecorations(state:EditorState,globalEnabled:()=>boolean):DecorationSet {
  const builder=new RangeSetBuilder<Decoration>();
  const source=state.doc.toString();
  if(!mathShorthandEnabled(source,globalEnabled())) return builder.finish();

  for(const range of scanMathRanges(source)){
    if(state.selection.ranges.some(selection=>selection.from<=range.to&&selection.to>=range.from)) continue;
    const raw=source.slice(range.contentFrom,range.contentTo);
    const expanded=expandMathShorthand(raw);
    if(expanded===raw) continue;

    const startsLine=state.doc.lineAt(range.from).from===range.from;
    const endsLine=state.doc.lineAt(range.to).to===range.to;
    builder.add(range.from,range.to,Decoration.replace({
      widget:new MathWidget(expanded,range.display),
      block:range.display&&startsLine&&endsLine,
    }));
  }
  return builder.finish();
}

/**
 * State fields may safely provide layout-changing block decorations. A view
 * plugin cannot do so because CodeMirror computes its viewport before running
 * view plugins, which caused display-math notes to fail to open.
 */
export function mathShorthandEditorExtension(enabled:()=>boolean){
  return Prec.high(StateField.define<DecorationSet>({
    create:state=>buildDecorations(state,enabled),
    update:(decorations,transaction)=>transaction.docChanged||transaction.selection
      ?buildDecorations(transaction.state,enabled)
      :decorations.map(transaction.changes),
    provide:field=>EditorView.decorations.from(field),
  }));
}
