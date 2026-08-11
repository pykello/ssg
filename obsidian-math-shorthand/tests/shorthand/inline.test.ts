import { describe,expect,it } from "vitest";
import { expandMathShorthand } from "../../src/shorthand/expand";
describe("SSG shorthand parity",()=>{it.each([
  ["v{x} + v{y}","\\mathbf{x} + \\mathbf{y}"],
  ["eps","\\epsilon"],
  ["norm(v{x} - v{y}) <= eps","\\left\\lVert \\mathbf{x} - \\mathbf{y} \\right\\rVert \\le \\epsilon"],
  ["x in bb{R}^n","x \\in \\mathbb{R}^n"],
  ["lim[x -> 0](f(x)) = 1","\\lim_{x \\to 0} f(x) = 1"],
  ["sum[i=1..n](a_i)","\\sum_{i=1}^{n} a_i"],
  ["int[a..b](f(x), x)","\\int_{a}^{b} f(x)\\,dx"],
  ["set(v{x} in bb{R}^n | norm(v{x}) <= 1)","\\left\\{\\mathbf{x} \\in \\mathbb{R}^n \\;\\middle|\\; \\left\\lVert \\mathbf{x} \\right\\rVert \\le 1\\right\\}"],
  ["\\gamma + norm(v{x})","\\gamma + \\left\\lVert \\mathbf{x} \\right\\rVert"],
  ["\\text{alpha in beta}","\\text{alpha in beta}"],
])("expands %s",(input,expected)=>expect(expandMathShorthand(input)).toBe(expected));});
