"""Incremental-workspace semantic index for the documented CSharpX subset.

This is deliberately not a full CSharpX parser. It models declarations, inheritance,
scopes, and name binding, and reports unsupported/ambiguous cases conservatively.
"""
from __future__ import annotations
from dataclasses import dataclass, field
import re
from pathlib import Path
from urllib.parse import unquote, urlparse

TOKEN = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"|\b[A-Za-z_]\w*\b|\d+|[^\s]', re.M)
MODS = {'public','private','protected','internal','static','virtual','override','abstract','sealed','new','unsafe','extern','async','readonly','partial'}
BUILTIN = {'void','int','long','short','byte','float','double','bool','string','char','object','var','decimal'}
KEYWORDS = MODS | BUILTIN | {'class','interface','struct','enum','namespace','using','return','if','else','while','for','foreach','try','catch','finally','throw','base','this','null','true','false','new','await'}
@dataclass(frozen=True)
class Token:
    value: str
    start: int
    end: int
@dataclass
class Symbol:
    name: str
    kind: str
    uri: str
    start: int
    end: int
    container: str = ''
    type_name: str = ''
    base: str = ''
    modifiers: set = field(default_factory=set)
    params: list = field(default_factory=list)
    scope_start: int = 0
    scope_end: int = 0
    def key(self): return (self.uri,self.start,self.end)

def tokenize(src):
    return [Token(m.group(),m.start(),m.end()) for m in TOKEN.finditer(src) if not m.group().startswith(('//','/*','"'))]

def offset(src,position):
    lines=src.splitlines(keepends=True)
    line=max(0,min(position.get('line',0),len(lines)))
    return min(len(src),sum(map(len,lines[:line]))+position.get('character',0))
def position(src,off):
    return {'line':src.count('\n',0,off),'character':off-(src.rfind('\n',0,off)+1)}
def range_(src,a,b):return {'start':position(src,a),'end':position(src,b)}

def matching(tokens,op,cl):
    depth=0
    for j in range(op,len(tokens)):
        if tokens[j].value=='{':depth+=1
        elif tokens[j].value=='}':
            depth-=1
            if depth==0:return j
    return len(tokens)-1

def parse(uri,src):
    ts=tokenize(src);symbols=[];issues=[]
    def add(tok,kind,**kw):
        sym=Symbol(tok.value,kind,uri,tok.start,tok.end,**kw);symbols.append(sym);return sym
    # Parse classes and their members, excluding nested method bodies.
    i=0
    while i<len(ts):
        t=ts[i]
        if t.value not in ('class','interface','struct','enum') or i+1>=len(ts):i+=1;continue
        name=ts[i+1];base='';j=i+2
        if j<len(ts) and ts[j].value==':':
            j+=1
            if j<len(ts):base=ts[j].value
        while j<len(ts) and ts[j].value not in ('{',';'):j+=1
        if j>=len(ts) or ts[j].value!='{':i+=1;continue
        close=matching(ts,j,len(ts));klass=add(name,'class',base=base,scope_start=ts[j].end,scope_end=ts[close].start)
        k=j+1
        while k<close:
            if ts[k].value in ('{','}'):k+=1;continue
            mods=set()
            while k<close and ts[k].value in MODS:mods.add(ts[k].value);k+=1
            if k+1>=close:break
            if ts[k].value in ('class','interface','struct','enum'):
                k+=1;continue
            typ=ts[k].value;member=ts[k+1]
            if not re.fullmatch(r'[A-Za-z_]\w*',member.value):k+=1;continue
            if k+2<close and ts[k+2].value=='(':
                end=k+3;params=[]
                while end<close and ts[end].value!=')':
                    if end+1<close and ts[end+1].value not in (',',')'):
                        params.append((ts[end].value,ts[end+1].value,ts[end+1].start,ts[end+1].end))
                        end+=2
                    else:end+=1
                    if end<close and ts[end].value==',':end+=1
                body=end+1
                while body<close and ts[body].value not in ('{',';'):body+=1
                body_end=matching(ts,body,len(ts)) if body<close and ts[body].value=='{' else body
                method=add(member,'method',container=klass.name,type_name=typ,modifiers=mods,params=params,scope_start=ts[body].end if body<close else member.end,scope_end=ts[body_end].start if body_end<len(ts) else member.end)
                for pt,pn,pa,pb in params:symbols.append(Symbol(pn,'parameter',uri,pa,pb,container=klass.name+'.'+member.value,type_name=pt,scope_start=method.scope_start,scope_end=method.scope_end))
                # Local declarations inside method bodies
                if body<close and ts[body].value=='{':
                    q=body+1
                    while q+1<body_end:
                        if ts[q].value in BUILTIN | {klass.name} or (ts[q].value[:1].isupper() and ts[q].value not in KEYWORDS):
                            if re.fullmatch(r'[A-Za-z_]\w*',ts[q+1].value) and (q+2<body_end and ts[q+2].value in ('=',';',',')):
                                symbols.append(Symbol(ts[q+1].value,'local',uri,ts[q+1].start,ts[q+1].end,container=klass.name+'.'+member.value,type_name=ts[q].value,scope_start=ts[q+1].end,scope_end=method.scope_end))
                        q+=1
                k=body_end+1;continue
            if k+2<close and ts[k+2].value in (';','=','{',','):
                add(member,'field',container=klass.name,type_name=typ,modifiers=mods)
                k+=2
                while k<close and ts[k].value!=';':k+=1
                k+=1;continue
            k+=1
        i=close+1
    # Top-level methods (e.g. prototype's void Main()).
    for k in range(len(ts)-3):
        if ts[k].value in BUILTIN and ts[k+2].value=='(' and ts[k+1].value not in KEYWORDS:
            name=ts[k+1]
            if any(s.start==name.start for s in symbols):continue
            if not any(s.kind=='class' and s.scope_start<=name.start<=s.scope_end for s in symbols):
                add(name,'method',type_name=ts[k].value)
    return symbols,issues

class Workspace:
    def __init__(self):self.docs={};self.symbols={};self.tokens={}
    def update(self,uri,text):
        self.docs[uri]=text;self.symbols[uri],_=parse(uri,text);self.tokens[uri]=tokenize(text)
    def remove(self,uri):
        for d in (self.docs,self.symbols,self.tokens):d.pop(uri,None)
    def load_project(self,root):
        path=Path(root)
        if path.is_file():path=path.parent
        if not path.is_dir():return
        for f in path.rglob('*.csx'):
            if any(p in ('bin','obj','.git') for p in f.parts):continue
            uri=f.resolve().as_uri()
            if uri not in self.docs:
                try:self.update(uri,f.read_text(encoding='utf-8'))
                except (OSError,UnicodeError):pass
    def all_symbols(self):return [s for syms in self.symbols.values() for s in syms]
    def class_named(self,name):return [s for s in self.all_symbols() if s.kind=='class' and s.name==name]
    def enclosing_class(self,uri,at):
        classes=[s for s in self.symbols.get(uri,[]) if s.kind=='class' and s.scope_start<=at<=s.scope_end]
        return min(classes,key=lambda s:s.scope_end-s.scope_start) if classes else None
    def enclosing_method(self,uri,at):
        methods=[s for s in self.symbols.get(uri,[]) if s.kind=='method' and s.scope_start<=at<=s.scope_end]
        return min(methods,key=lambda s:s.scope_end-s.scope_start) if methods else None
    def members(self,class_name):
        result=[];seen=set();visited=set();current=class_name
        while current and current not in visited:
            visited.add(current);matches=self.class_named(current)
            if len(matches)!=1:break
            for s in self.all_symbols():
                if s.container==current and s.kind in ('field','method') and (s.name,s.kind) not in seen:
                    seen.add((s.name,s.kind));result.append(s)
            current=matches[0].base
        return result
    def symbol_at(self,uri,at):
        tok=next((t for t in self.tokens.get(uri,[]) if t.start<=at<t.end),None)
        if tok is None and at>0:tok=next((t for t in self.tokens.get(uri,[]) if t.start<=at-1<t.end),None)
        return tok
    def resolve(self,uri,at):
        tok=self.symbol_at(uri,at)
        if tok is None or not tok.value.isidentifier():return []
        ts=self.tokens.get(uri,[]);idx=next((i for i,t in enumerate(ts) if t.start==tok.start),-1)
        if idx>=2 and ts[idx-1].value=='.':
            receiver=ts[idx-2].value;owner=self.enclosing_class(uri,at);method=self.enclosing_method(uri,at)
            typ=owner.name if receiver=='this' and owner else owner.base if receiver=='base' and owner else ''
            if not typ:
                locals_=[s for s in self.symbols.get(uri,[]) if s.name==receiver and s.kind in ('local','parameter') and s.scope_start<=at<=s.scope_end]
                if locals_:typ=max(locals_,key=lambda s:s.start).type_name
            return [s for s in self.members(typ) if s.name==tok.value] if typ else []
        declarations=[s for s in self.symbols.get(uri,[]) if s.start==tok.start and s.end==tok.end]
        if declarations:return declarations
        local=[s for s in self.symbols.get(uri,[]) if s.name==tok.value and s.kind in ('local','parameter') and s.scope_start<=at<=s.scope_end and (s.kind=='parameter' or s.start<=at)]
        if local:return [max(local,key=lambda s:s.start)]
        owner=self.enclosing_class(uri,at)
        if owner:
            members=[s for s in self.members(owner.name) if s.name==tok.value]
            if members:return members
        return [s for s in self.all_symbols() if s.name==tok.value and s.kind=='class'] or [s for s in self.all_symbols() if s.name==tok.value and s.kind=='method' and not s.container]
    def completions(self,uri,at):
        src=self.docs.get(uri,'');prefix=src[:at];match=re.search(r'([A-Za-z_]\w*|this|base)\.\s*[A-Za-z_]*$',prefix)
        if match:
            receiver=match.group(1);owner=self.enclosing_class(uri,at);typ=''
            if owner and receiver in ('this','base'):typ=owner.name if receiver=='this' else owner.base
            else:
                local=[s for s in self.symbols.get(uri,[]) if s.name==receiver and s.kind in ('local','parameter') and s.scope_start<=at<=s.scope_end]
                if local:typ=max(local,key=lambda s:s.start).type_name
            return self.members(typ) if typ else []
        return [s for s in self.all_symbols() if s.kind=='class']+[s for s in self.symbols.get(uri,[]) if s.kind in ('local','parameter','method','field') and (s.kind not in ('local','parameter') or s.scope_start<=at<=s.scope_end)]
    def diagnostics(self,uri):
        src=self.docs.get(uri,'');ts=self.tokens.get(uri,[]);out=[];stack=[]
        def issue(a,b,code,message):out.append({'range':range_(src,a,b),'severity':1,'source':'CSharpX semantic subset','code':code,'message':message})
        for t in ts:
            if t.value=='{':stack.append(t)
            elif t.value=='}':
                if stack:stack.pop()
                else:issue(t.start,t.end,'CSX1001','Unmatched closing brace')
        for t in stack:issue(t.start,t.end,'CSX1002','Unmatched opening brace')
        classes=[s for s in self.all_symbols() if s.kind=='class']
        for s in self.symbols.get(uri,[]):
            if s.kind!='class':continue
            if s.base and not self.class_named(s.base):issue(s.start,s.end,'CSX2202',f'Unknown base class {s.base}')
            visited=set();cur=s
            while cur and cur.name not in visited:
                visited.add(cur.name)
                parent=self.class_named(cur.base) if cur.base else []
                cur=parent[0] if len(parent)==1 else None
            if cur:issue(s.start,s.end,'CSX2203',f'Inheritance cycle involving {s.name}')
        for s in self.symbols.get(uri,[]):
            if s.kind=='method' and 'override' in s.modifiers:
                owner=self.class_named(s.container)
                inherited=self.members(owner[0].base) if len(owner)==1 and owner[0].base else []
                if not any(m.name==s.name and ('virtual' in m.modifiers or 'override' in m.modifiers or 'abstract' in m.modifiers) for m in inherited):
                    issue(s.start,s.end,'CSX2205',f'Invalid override {s.container}.{s.name}')
        return out
    def occurrences(self,symbol):
        result=[]
        for uri,ts in self.tokens.items():
            for t in ts:
                if t.value!=symbol.name:continue
                candidates=self.resolve(uri,t.start)
                if any(s.key()==symbol.key() for s in candidates):result.append((uri,t.start,t.end))
        return result
