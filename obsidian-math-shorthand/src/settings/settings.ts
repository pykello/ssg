import { App, Notice, PluginSettingTab, Setting } from "obsidian";
import type MathShorthandPlugin from "../main";

export interface MathShorthandSettings { enabled:boolean; }
export const DEFAULT_SETTINGS:MathShorthandSettings={enabled:true};
export class MathShorthandSettingTab extends PluginSettingTab {
  constructor(app:App,private plugin:MathShorthandPlugin){super(app,plugin);}
  display(){this.containerEl.empty();new Setting(this.containerEl).setName("Enable math shorthand").setDesc("Virtually expand shorthand in Live Preview and Reading View.").addToggle(toggle=>toggle.setValue(this.plugin.settings.enabled).onChange(async value=>{const previous=this.plugin.settings.enabled;this.plugin.settings.enabled=value;try{await this.plugin.saveData(this.plugin.settings);this.plugin.refreshViews();}catch(error){this.plugin.settings.enabled=previous;toggle.setValue(previous);console.error("Failed to save Math Shorthand settings",error);new Notice("Could not save the Math Shorthand setting.");}}));}
}
