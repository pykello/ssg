import type { CodeFence, MathRange } from "./types";

function escaped(source: string, at: number): boolean { let n=0; for(let i=at-1;i>=0&&source[i]==="\\";i--) n++; return n%2===1; }

/** Finds dollar math structurally, ignoring escaped dollars and fenced code. */
export function scanMathRanges(source: string): MathRange[] {
  const ranges: MathRange[]=[]; let i=0; let fence: CodeFence|undefined;
  while(i<source.length){
    const lineStart=i===0||source[i-1]==="\n";
    if(lineStart){
      const lineEnd=source.indexOf("\n",i);
      const line=source.slice(i,lineEnd<0?source.length:lineEnd);
      const match=/^\s*(`{3,}|~{3,})/.exec(line);
      if(match){
        const run=match[1]!;
        const marker=run[0] as CodeFence["marker"];
        if(!fence) fence={marker,length:run.length};
        else if(marker===fence.marker&&run.length>=fence.length) fence=undefined;
        i=lineEnd<0?source.length:lineEnd+1;
        continue;
      }
    }
    if(!fence&&source[i]==="$"&&!escaped(source,i)){
      const display=source[i+1]==="$"; const delimiter=display?"$$":"$"; const start=i; i+=delimiter.length;
      let end=-1; while(i<source.length){ if(source.startsWith(delimiter,i)&&!escaped(source,i)){end=i;break;} if(!display&&source[i]==="\n") break; i++; }
      if(end>=0){ranges.push({from:start,to:end+delimiter.length,contentFrom:start+delimiter.length,contentTo:end,display});i=end+delimiter.length;continue;}
    }
    i++;
  } return ranges;
}

export function mathShorthandEnabled(source:string, fallback=true):boolean {
  let result=fallback; const re=/<!--\s*ssg-math-shorthand:\s*(on|off|true|false|yes|no)\s*-->/g; for(const m of source.matchAll(re)) result=/^(on|true|yes)$/.test(m[1]!); return result;
}
