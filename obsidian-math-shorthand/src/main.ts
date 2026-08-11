import { MarkdownView, Plugin } from "obsidian";
import type { Extension } from "@codemirror/state";
import { mathShorthandEditorExtension } from "./editor/extension";
import { processMathShorthand } from "./preview/postprocessor";
import { DEFAULT_SETTINGS, MathShorthandSettings, MathShorthandSettingTab } from "./settings/settings";

export default class MathShorthandPlugin extends Plugin {
  settings:MathShorthandSettings=DEFAULT_SETTINGS;
  private readonly editorExtensions:Extension[]=[];
  async onload(){this.settings=Object.assign({},DEFAULT_SETTINGS,await this.loadData());this.updateEditorExtension();this.registerEditorExtension(this.editorExtensions);this.registerMarkdownPostProcessor((el,ctx)=>processMathShorthand(el,ctx,this.settings.enabled));this.addSettingTab(new MathShorthandSettingTab(this.app,this));}
  private updateEditorExtension(){this.editorExtensions.splice(0,this.editorExtensions.length,mathShorthandEditorExtension(()=>this.settings.enabled));}
  refreshViews(){for(const leaf of this.app.workspace.getLeavesOfType("markdown"))leaf.view instanceof MarkdownView&&leaf.view.previewMode.rerender(true);this.updateEditorExtension();this.app.workspace.updateOptions();}
}
