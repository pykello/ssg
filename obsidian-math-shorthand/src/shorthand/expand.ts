const protectedCommands=["text","textrm","textit","textbf","mathrm","operatorname","mbox"];
const wrapped:Record<string,[string,string]>={norm:["\\left\\lVert "," \\right\\rVert"],abs:["\\left\\lvert "," \\right\\rvert"],unit:["\\hat{\\mathbf{","}}"],v:["\\mathbf{","}"],bb:["\\mathbb{","}"],cal:["\\mathcal{","}"],hat:["\\hat{","}"]};
const words:Record<string,string>={subseteq:"\\subseteq",supseteq:"\\supseteq",forall:"\\forall",exists:"\\exists",notin:"\\notin",subset:"\\subset",supset:"\\supset",union:"\\cup",inter:"\\cap",in:"\\in",dot:"\\cdot",cross:"\\times"};
const greek=["alpha","beta","gamma","delta","eta","theta","lambda","mu","xi","pi","rho","sigma","tau","phi","psi","omega","Delta","Gamma","Omega","Phi","Psi"];
const indexed:Record<string,string>={sum:"\\sum",prod:"\\prod",lim:"\\lim",sup:"\\sup",inf:"\\inf",max:"\\max",min:"\\min",union:"\\bigcup",inter:"\\bigcap"};
const integrals:Record<string,string>={iiint:"\\iiint",iint:"\\iint",oint:"\\oint",int:"\\int"};
function escaped(s:string,i:number){let n=0;for(i--;i>=0&&s[i]==="\\";i--)n++;return n%2===1;}
function matching(s:string,open:number,a:string,b:string){let depth=0;for(let i=open;i<s.length;i++){if(escaped(s,i))continue;if(s[i]===a)depth++;else if(s[i]===b&&--depth===0)return i;}return -1;}
function boundary(s:string,i:number,n:string){const word=(c:string|undefined)=>!!c&&/[A-Za-z0-9_]/.test(c);return !escaped(s,i)&&!word(s[i-1])&&!word(s[i+n.length]);}
function calls(s:string,name:string,open:string,close:string,fn:(body:string,index?:string)=>string):string{let out="";for(let i=0;i<s.length;){if(s.startsWith(name+open,i)&&boundary(s,i,name)){const end=matching(s,i+name.length,open,close);if(end>=0){out+=fn(s.slice(i+name.length+1,end));i=end+1;continue;}}out+=s[i++];}return out;}
function splitTop(s:string,separator:string):string[]{let p=0,b=0,q=0,start=0,out:string[]=[];for(let i=0;i<s.length;i++){if(escaped(s,i))continue;const c=s[i];if(c==="(")p++;else if(c===")")p--;else if(c==="{")b++;else if(c==="}")b--;else if(c==="[")q++;else if(c==="]")q--;else if(!p&&!b&&!q&&s.startsWith(separator,i)){out.push(s.slice(start,i).trim());start=i+separator.length;i+=separator.length-1;}}out.push(s.slice(start).trim());return out;}
function args(s:string){return splitTop(s,",").map(expandMathShorthand);}
function parenFns(s:string,names:string[],fn:(n:string,a:string[])=>string|undefined){for(const n of names)s=calls(s,n,"(",")",body=>fn(n,args(body))??`${n}(${body})`);return s;}
function protect(s:string){const values:string[]=[];let out="";for(let i=0;i<s.length;){let found=false;for(const n of protectedCommands){const prefix=`\\${n}{`;if(s.startsWith(prefix,i)&&!escaped(s,i)){const end=matching(s,i+prefix.length-1,"{","}");if(end>=0){out+=`ZZMATHLITERAL${values.length}ZZ`;values.push(s.slice(i,end+1));i=end+1;found=true;break;}}}if(!found)out+=s[i++];}return {out,restore:(v:string)=>values.reduce((x,value,i)=>x.replaceAll(`ZZMATHLITERAL${i}ZZ`,value),v)};}
function binder(s:string){const range=splitTop(s,"..");if(range.length===2)return `_{${range[0]}}^{${range[1]}}`;const arrow=splitTop(s,"->");if(arrow.length===2)return `_{${arrow[0]} \\to ${arrow[1]}}`;const membership=splitTop(s," in ");if(membership.length===2)return `_{${membership[0]} \\in ${membership[1]}}`;return `_{${expandMathShorthand(s.trim())}}`;}
function indexedCalls(s:string,name:string,latex:string):string {let out="";for(let i=0;i<s.length;){if(s.startsWith(name+"[",i)&&boundary(s,i,name)){const end=matching(s,i+name.length,"[","]");if(end>=0){let rendered=latex+binder(s.slice(i+name.length+1,end));let next=end+1;if(s[next]==="("){const bodyEnd=matching(s,next,"(",")");if(bodyEnd>=0){rendered+=` ${expandMathShorthand(s.slice(next+1,bodyEnd))}`;next=bodyEnd+1;}}out+=rendered;i=next;continue;}}out+=s[i++];}return out;}

/** Structural, recursive port of the SSG shorthand expander. */
export function expandMathShorthand(source:string):string {
  const protectedText=protect(source); let s=protectedText.out;
  for(const [name,latex] of Object.entries(indexed)) s=indexedCalls(s,name,latex);
  for(const [name,latex] of Object.entries(integrals)) s=calls(s,name,"[","]",bound=>`${latex}${binder(bound)}`);
  s=s.replace(/(\\(?:i{1,3}nt|oint)(?:_\{[^{}]*\})?(?:\^\{[^{}]*\})?)\(([^()]*)\)/g,(_,op,body)=>{const a=splitTop(body,",");return `${op} ${expandMathShorthand(a[0]??"")}${a.slice(1).map(x=>`\\,d${expandMathShorthand(x)}`).join("")}`;});
  s=parenFns(s,["sum","prod"],(n,a)=>`${indexed[n]} ${a.join(", ")}`);
  for(const [n,[a,b]] of Object.entries(wrapped)) {s=calls(s,n,"{","}",x=>a+expandMathShorthand(x)+b);s=calls(s,n,"(",")",x=>a+expandMathShorthand(x)+b);}
  s=calls(s,"set","(",")",x=>{let z=splitTop(x,"|");if(z.length===1)z=splitTop(x,":");return `\\left\\{${z.length===2?`${expandMathShorthand(z[0]!)} \\;\\middle|\\; ${expandMathShorthand(z[1]!)}`:expandMathShorthand(x)}\\right\\}`;});
  s=parenFns(s,["img","pre","comp","cl","interior","bd","ball","openball","closedball"],(n,a)=>({img:a.length===2?`${a[0]}(${a[1]})`:undefined,pre:a.length===2?`${a[0]}^{-1}(${a[1]})`:undefined,comp:a.length===1?`{${a[0]}}^c`:undefined,cl:a.length===1?`\\overline{${a[0]}}`:undefined,interior:a.length===1?`{${a[0]}}^\\circ`:undefined,bd:a.length===1?`\\partial ${a[0]}`:undefined,ball:a.length===2?`B_{${a[1]}}(${a[0]})`:undefined,openball:a.length===2?`B_{${a[1]}}(${a[0]})`:undefined,closedball:a.length===2?`\\overline{B}_{${a[1]}}(${a[0]})`:undefined})[n]);
  s=parenFns(s,["dd","pd","pd2","grad","div","curl","jac","hess"],(n,a)=>{if(n==="dd"&&a.length===2)return `\\frac{d ${a[0]}}{d ${a[1]}}`;if(n==="pd"&&a.length===2)return `\\frac{\\partial ${a[0]}}{\\partial ${a[1]}}`;if(n==="pd2"&&a.length===3)return `\\frac{\\partial^2 ${a[0]}}{\\partial ${a[1]} \\partial ${a[2]}}`;if(a.length===1)return ({grad:`\\nabla ${a[0]}`,div:`\\operatorname{div} ${a[0]}`,curl:`\\operatorname{curl} ${a[0]}`,jac:`J_{${a[0]}}`,hess:`H_{${a[0]}}`})[n];});
  s=parenFns(s,["mat","pmat","detmat"],(n,a)=>{const rows=splitTop(a.join(","),";").map(r=>splitTop(r,",").map(expandMathShorthand).join(" & ")).join(" \\\\ ");const env=n==="pmat"?"pmatrix":n==="detmat"?"vmatrix":"bmatrix";return `\\begin{${env}}${rows}\\end{${env}}`;});
  s=parenFns(s,["wedge","ext","pull","form","boundary"],(n,a)=>({wedge:a.join(" \\wedge "),ext:a.length===1?`d{${a[0]}}`:undefined,pull:a.length===2?`${a[0]}^*${a[1]}`:undefined,form:a.length===1?`\\lambda_{${a[0]}}`:undefined,boundary:a.length===1?`\\partial ${a[0]}`:undefined})[n]);
  const operators:Array<[string,string]>=[["...","\\ldots"],["<=>","\\Leftrightarrow"],["=>","\\implies"],["->","\\to"],["!=","\\ne"],["<=","\\le"],[">=","\\ge"]];
  for(const [from,to] of operators)s=s.split(from).join(to);
  for(const [from,to] of Object.entries(words))s=s.replace(new RegExp(`(?<![\\\\A-Za-z0-9_])${from}(?![A-Za-z0-9_])`,"g"),to);
  for(const name of greek)s=s.replace(new RegExp(`(?<![\\\\A-Za-z0-9_])${name}(?![A-Za-z0-9_])`,"g"),`\\${name}`);
  for(const [name,to] of Object.entries({eps:"\\epsilon",del:"\\delta",inf:"\\infty"}))s=s.replace(new RegExp(`(?<![\\\\A-Za-z0-9_])${name}(?=$|[_^]|[^A-Za-z0-9_])`,"g"),to);
  return protectedText.restore(s);
}
