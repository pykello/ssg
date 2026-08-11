import { MarkdownView, Plugin } from "obsidian";
import { mathShorthandEditorExtension } from "./editor/extension";
import { processMathShorthand } from "./preview/postprocessor";
import { DEFAULT_SETTINGS, MathShorthandSettings, MathShorthandSettingTab } from "./settings/settings";

export default class MathShorthandPlugin extends Plugin {
  settings:MathShorthandSettings=DEFAULT_SETTINGS;
  async onload(){this.settings=Object.assign({},DEFAULT_SETTINGS,await this.loadData());this.registerEditorExtension(mathShorthandEditorExtension(()=>this.settings.enabled));this.registerMarkdownPostProcessor((el,ctx)=>processMathShorthand(el,ctx,this.settings.enabled));this.addSettingTab(new MathShorthandSettingTab(this.app,this));}
  refreshViews(){for(const leaf of this.app.workspace.getLeavesOfType("markdown"))leaf.view instanceof MarkdownView&&leaf.view.previewMode.rerender(true);this.app.workspace.updateOptions();}
}
