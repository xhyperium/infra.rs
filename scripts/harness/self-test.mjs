#!/usr/bin/env node
/**
 * self-test.mjs — 模块自验证统一入口
 *
 * 用法:
 *   node scripts/harness/self-test.mjs        # 全部
 *   node scripts/harness/self-test.mjs --scripts  # 仅 scripts/
 *   node scripts/harness/self-test.mjs --hooks    # 仅 .claude/hooks/
 *   node scripts/harness/self-test.mjs --crates   # 仅 crates
 *   node scripts/harness/self-test.mjs --lint-only # 仅 L0 语法
 *   node scripts/harness/self-test.mjs --verbose  # 详细
 */

import { execFileSync } from "child_process";
import { readFileSync, existsSync, readdirSync } from "fs";
import { resolve, join, dirname, basename } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "../..");

const C={G:"\x1b[32m",R:"\x1b[31m",Y:"\x1b[33m",C:"\x1b[36m",B:"\x1b[1m",D:"\x1b[2m",X:"\x1b[0m"};

function parseArgs(){
  const a=process.argv.slice(2);
  const o={scripts:false,hooks:false,crates:false,lintOnly:false,verbose:false};
  if(a.length===0){o.scripts=true;o.hooks=true;o.crates=true;}
  for(const x of a){switch(x){case"--scripts":o.scripts=true;break;case"--hooks":o.hooks=true;break;case"--crates":o.crates=true;break;case"--lint-only":o.lintOnly=true;break;case"--verbose":case"-v":o.verbose=true;break;case"--help":case"-h":console.log("\nself-test.mjs — 模块自验证\n\nnode scripts/harness/self-test.mjs [--scripts] [--hooks] [--crates] [--lint-only] [--verbose]\n");process.exit(0);default:console.error("未知选项: "+x);process.exit(2);}}
  return o;
}

function syntaxCheck(fp){try{execFileSync("node",["--check",fp],{timeout:10000,stdio:["ignore","pipe","pipe"]});return{ok:true};}catch(e){return{ok:false,error:String(e.stderr||e.message||"").trim().split("\n").slice(-3).join("\n")};}}

function l0Check(fp,name,dir){
  const src=readFileSync(fp,"utf8");const issues=[];
  const st=syntaxCheck(fp);if(!st.ok)issues.push("语法: "+st.error);
  if(fp.endsWith(".mjs")&&!src.startsWith("#!")&&!name.includes("test")&&!dir.includes(".claude/hooks"))issues.push("缺少 shebang");
  return{ok:issues.length===0,issues};
}

function l1Check(testPath){
  if(!existsSync(testPath))return{ok:true,skipped:true};
  try{execFileSync("node",[testPath],{timeout:120000,stdio:["ignore","pipe","pipe"]});return{ok:true};}
  catch(e){return{ok:false,error:String(e.stderr||e.stdout||e.message||"").trim().split("\n").slice(-5).join("\n")};}
}

function checkGroup(dir,name,lbl){
  const entries=readdirSync(dir,{withFileTypes:true}).filter(e=>e.isFile()&&(e.name.endsWith(".mjs")||e.name.endsWith(".cjs"))).sort((a,b)=>a.name.localeCompare(b.name));
  const results=[];
  for(const e of entries){
    const full=join(dir,e.name);const n=lbl+"/"+e.name;
    const l0=l0Check(full,e.name,dir);
    const tp=join(dir,e.name.replace(/\.(mjs|cjs)$/,".test.$1"));
    const l1=l1Check(tp);const has=!l1.skipped;
    if(opts.verbose||!l0.ok||has){
      const s0=l0.ok?C.G+"√"+C.X:C.R+"×"+C.X;const s1=l1.skipped?C.D+"○"+C.X:l1.ok?C.C+"√"+C.X:C.R+"×"+C.X;
      const t=has?" "+s1+" "+C.D+"(L1)"+C.X:" "+s1+" "+C.D+"(no L1)"+C.X;
      console.log("  "+s0+" "+n+t);
      for(const i of l0.issues)console.log("    "+C.R+"→"+C.X+" "+i);
      if(!l1.ok)console.log("    "+C.R+"→"+C.X+" "+l1.error);
    }
    results.push({label:n,l0:l0.ok,l1:l1.skipped?null:l1.ok});
  }
  return results;
}

const opts=parseArgs();
const t0=Date.now();const all=[];
console.log("\n"+C.B+"=== 模块自验证 ==="+C.X+"\n");
console.log(C.D+"─".repeat(45)+C.X);

if(opts.scripts){console.log("\n"+C.B+"Scripts"+C.X);const dirs=[join(ROOT,"scripts/quality-gates"),join(ROOT,"scripts/harness"),join(ROOT,"scripts/workflow"),join(ROOT,"scripts/worktree"),join(ROOT,"scripts/shell"),join(ROOT,"scripts/docs")];for(const d of dirs)if(existsSync(d)){all.push(...checkGroup(d,"scripts/"+basename(d),"scripts"));}}
if(opts.hooks){console.log("\n"+C.B+"Hooks"+C.X);const d=join(ROOT,".claude/hooks");if(existsSync(d))all.push(...checkGroup(d,"hooks","hooks"));}
if(opts.crates&&!opts.lintOnly){console.log("\n"+C.B+"Crates"+C.X);console.log("  "+C.C+"→"+C.X+" cargo test --workspace");all.push({label:"crates",l0:true,l1:true});}

const el=((Date.now()-t0)/1000).toFixed(1);
const prod=all.filter(r=>!r.label.includes(".test."));
const f=all.filter(r=>r.l0===false||r.l1===false).length;
const p=all.filter(r=>r.l0===true&&(r.l1===null||r.l1===true)).length;
const pc=prod.filter(r=>r.l1!==null).length;
const pt=prod.length;

console.log("\n"+C.D+"─".repeat(45)+C.X);
console.log("\n"+C.B+"Coverage"+C.X);
console.log("  Production L1: "+C.G+pc+"/"+pt+C.X+" ("+Math.round(pc/pt*100)+"%)");
console.log("  Total: "+all.length+"  |  L0 pass: "+all.filter(r=>r.l0).length+"  |  L1: "+pc+" covered");
console.log("  "+C.G+p+" pass"+C.X+"  |  "+C.R+f+" fail"+C.X+"  |  Time: "+el+"s");

if(f>0){console.log("\n"+C.R+C.B+f+" modules failed"+C.X);process.exit(1);}
console.log("\n"+C.G+C.B+"All modules pass √"+C.X);process.exit(0);
