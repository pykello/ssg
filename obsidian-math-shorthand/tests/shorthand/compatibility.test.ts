import { expect,it } from "vitest";import { mathShorthandEnabled,scanMathRanges } from "../../src/shorthand/scanner";
it("uses the last SSG directive",()=>expect(mathShorthandEnabled("<!-- ssg-math-shorthand: on -->\n<!-- ssg-math-shorthand: off -->",true)).toBe(false));
it("ignores escaped dollars and code fences",()=>expect(scanMathRanges("\\$no$\n```\n$x$\n```\n$yes$ ")).toEqual([{from:18,to:23,contentFrom:19,contentTo:22,display:false}]));
it("only closes a fence with the same marker and sufficient length",()=>expect(scanMathRanges("````\n$x$\n```\n~~~\n$y$\n````\n$yes$")).toEqual([{from:26,to:31,contentFrom:27,contentTo:30,display:false}]));
it("finds single-line and multiline display math",()=>expect(scanMathRanges("$$eps$$\n$$\nnorm(v{x})\n$$")).toEqual([
  {from:0,to:7,contentFrom:2,contentTo:5,display:true},
  {from:8,to:24,contentFrom:10,contentTo:22,display:true},
]));
