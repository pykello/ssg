import { expect,it } from "vitest";
import { sectionSource } from "../../src/shorthand/scanner";

it("extracts only the source represented by a Reading View section",()=>{
  expect(sectionSource("before $alpha$\nsection $eps$\nafter $beta$",1,1)).toBe("section $eps$");
});
