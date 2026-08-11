import { expect,it } from "vitest";import { mathShorthandEnabled,scanMathRanges } from "../../src/shorthand/scanner";
it("uses the last SSG directive",()=>expect(mathShorthandEnabled("<!-- ssg-math-shorthand: on -->\n<!-- ssg-math-shorthand: off -->",true)).toBe(false));
it("ignores escaped dollars and code fences",()=>expect(scanMathRanges("\\$no$\n```\n$x$\n```\n$yes$ ")).toEqual([{from:18,to:23,contentFrom:19,contentTo:22,display:false}]));
