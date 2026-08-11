import { WidgetType } from "@codemirror/view";
import { finishRenderMath, renderMath } from "obsidian";
export class MathWidget extends WidgetType {
  constructor(private latex:string,private display:boolean){super();}
  eq(other:MathWidget){return this.latex===other.latex&&this.display===other.display;}
  toDOM(){const el=renderMath(this.latex,this.display);el.addClass("math-shorthand-rendered");finishRenderMath();return el;}
  ignoreEvent(){return false;}
}
