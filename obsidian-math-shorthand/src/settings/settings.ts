import { App, PluginSettingTab, Setting } from "obsidian";
import type MathShorthandPlugin from "../main";

export interface MathShorthandSettings { enabled:boolean; }
export const DEFAULT_SETTINGS:MathShorthandSettings={enabled:true};
export class MathShorthandSettingTab extends PluginSettingTab {
  constructor(app:App,private plugin:MathShorthandPlugin){super(app,plugin);}
  display(){this.containerEl.empty();new Setting(this.containerEl).setName("Enable math shorthand").setDesc("Virtually expand shorthand in Live Preview and Reading View.").addToggle(toggle=>toggle.setValue(this.plugin.settings.enabled).onChange(async value=>{this.plugin.settings.enabled=value;await this.plugin.saveData(this.plugin.settings);this.plugin.refreshViews();}));}
}
