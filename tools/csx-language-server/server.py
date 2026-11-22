#!/usr/bin/env python3
"""CSharpX prototype LSP server (stdlib only). Features: completion, definition,
references, hover, document symbols, diagnostics, signature help and rename.
The parser is intentionally a subset, not a full compiler semantic engine.
"""
import json, re, sys, subprocess, os, pathlib, urllib.parse
from semantic import Workspace, range_, offset
KEYWORDS = 'class interface struct enum public private protected internal static virtual override abstract sealed new return void int long float double bool string true false null if else while for foreach using namespace try catch finally throw unsafe extern base this var async await'.split()
TYPES = ['Console','Exception','Sensor','List','Dictionary','string','int','bool','float','double']
DECL = re.compile(r'\b(?:(?:public|private|protected|internal|static|virtual|override|abstract|sealed|unsafe|extern|async)\s+)*(?:(class|interface|struct|enum)\s+(\w+)|(?:void|int|long|float|double|bool|string|var|[A-Z]\w*(?:<[^>]+>)?)\s+(\w+)\s*(?=\(|[=;,]))')
IDENT = re.compile(r'\b[A-Za-z_]\w*\b')

def pos(text, offset):
    before=text[:offset]; return {'line':before.count('\n'),'character':len(before.rsplit('\n',1)[-1])}
def off(text, p):
    lines=text.splitlines(keepends=True); return sum(len(x) for x in lines[:p['line']])+p['character']
def rng(text,a,b): return {'start':pos(text,a),'end':pos(text,b)}
def word(text, p):
    at=off(text,p)
    for m in IDENT.finditer(text):
        if m.start()<=at<=m.end():return m.group(),m.start(),m.end()
    return '',at,at

def declarations(text):
    result=[]
    for m in DECL.finditer(text):
        group=2 if m.group(2) else 3
        name=m.group(group)
        result.append((name,m.start(group),m.end(group), 5 if m.group(2) else 6))
    return result

def diagnostics(text):
    out=[]; stack=[]
    for m in re.finditer(r'[{}]',text):
        if m.group()=='{': stack.append(m.start())
        elif stack:stack.pop()
        else:out.append({'range':rng(text,m.start(),m.end()),'severity':1,'source':'CSharpX LSP','code':'CSX1001','message':'Unmatched closing brace'})
    for a in stack:out.append({'range':rng(text,a,a+1),'severity':1,'source':'CSharpX LSP','code':'CSX1002','message':'Unmatched opening brace'})
    return out

class Server:
    def __init__(self):self.docs={};self.workspace=Workspace();self.shutdown=False
    def send(self,obj):
        data=json.dumps(obj,separators=(',',':'),ensure_ascii=False).encode('utf-8');sys.stdout.buffer.write(b'Content-Length: '+str(len(data)).encode()+b'\r\n\r\n'+data);sys.stdout.buffer.flush()
    def reply(self,id,result=None,error=None):
        r={'jsonrpc':'2.0','id':id};r['error' if error else 'result']=error if error else result;self.send(r)
    def publish(self,uri):self.send({'jsonrpc':'2.0','method':'textDocument/publishDiagnostics','params':{'uri':uri,'diagnostics':self.workspace.diagnostics(uri)}})
    def handle(self,msg):
        method=msg.get('method');params=msg.get('params') or {};id=msg.get('id')
        if method=='initialize':
            
            for folder in params.get('workspaceFolders') or []:
                u=urllib.parse.urlparse(folder['uri']);self.workspace.load_project(urllib.parse.unquote(u.path).lstrip('/') if os.name=='nt' else urllib.parse.unquote(u.path))
            if params.get('rootUri'):
                u=urllib.parse.urlparse(params['rootUri']);self.workspace.load_project(urllib.parse.unquote(u.path).lstrip('/') if os.name=='nt' else urllib.parse.unquote(u.path))
            self.reply(id,{'capabilities':{'textDocumentSync':1,'completionProvider':{'triggerCharacters':['.']},'definitionProvider':True,'referencesProvider':True,'hoverProvider':True,'documentSymbolProvider':True,'renameProvider':True,'signatureHelpProvider':{'triggerCharacters':['(', ',']}} ,'serverInfo':{'name':'CSharpX Prototype LSP','version':'0.2'}});return
        if method=='shutdown':self.shutdown=True;self.reply(id);return
        if method=='exit':raise SystemExit(0 if self.shutdown else 1)
        if method=='textDocument/didOpen':
            d=params['textDocument'];self.docs[d['uri']]=d['text'];self.workspace.update(d['uri'],d['text']);self.publish(d['uri']);return
        if method=='textDocument/didChange':
            uri=params['textDocument']['uri'];self.docs[uri]=params['contentChanges'][-1]['text'];self.workspace.update(uri,self.docs[uri]);self.publish(uri);return
        if method=='textDocument/didClose':
            uri=params['textDocument']['uri'];self.docs.pop(uri,None);self.workspace.remove(uri);self.send({'jsonrpc':'2.0','method':'textDocument/publishDiagnostics','params':{'uri':uri,'diagnostics':[]}});return
        if id is None:return
        uri=params.get('textDocument',{}).get('uri','');text=self.docs.get(uri,'');position=params.get('position',{'line':0,'character':0});name,a,b=word(text,position)
        decls=declarations(text)
        at=offset(text,position)
        resolved=self.workspace.resolve(uri,at)
        def location(sym):
            source=self.workspace.docs[sym.uri]
            return {'uri':sym.uri,'range':range_(source,sym.start,sym.end)}
        if method=='textDocument/completion':
            symbols=self.workspace.completions(uri,at)
            items=[{'label':s.name,'kind':7 if s.kind=='class' else 2 if s.kind=='method' else 5,'detail':(s.container+'.' if s.container else '')+s.type_name} for s in symbols]
            if not re.search(r'\.\s*[A-Za-z_]*$',text[:at]):
                items.extend({'label':x,'kind':14,'detail':'CSharpX keyword'} for x in KEYWORDS)
            self.reply(id,{'isIncomplete':False,'items':list({(x['label'],x['detail']):x for x in items}.values())});return
        if method=='textDocument/definition':
            self.reply(id,[location(s) for s in resolved]);return
        if method=='textDocument/references':
            locations=[]
            for sym in resolved:
                for u,a,b in self.workspace.occurrences(sym):locations.append({'uri':u,'range':range_(self.workspace.docs[u],a,b)})
            self.reply(id,locations);return
        if method=='textDocument/hover':
            if resolved:
                sym=resolved[0]
                description=f'**{sym.kind}** `{sym.container+"." if sym.container else ""}{sym.name}`'
                if sym.type_name:description+=f' : `{sym.type_name}`'
                if sym.base:description+=f' : `{sym.base}`'
                self.reply(id,{'contents':{'kind':'markdown','value':description}})
            else:self.reply(id,None)
            return
        if method=='textDocument/documentSymbol':
            kinds={'class':5,'method':6,'field':8,'parameter':13,'local':13}
            self.reply(id,[{'name':s.name,'kind':kinds[s.kind],'range':range_(text,s.start,s.end),'selectionRange':range_(text,s.start,s.end)} for s in self.workspace.symbols.get(uri,[]) if s.kind in ('class','method','field')]);return
        if method=='textDocument/rename':
            new=params.get('newName','')
            if not re.fullmatch(r'[A-Za-z_]\w*',new) or new in KEYWORDS:
                self.reply(id,error={'code':-32602,'message':'Invalid CSharpX identifier'});return
            if len(resolved)!=1:
                self.reply(id,error={'code':-32602,'message':'Symbol cannot be resolved uniquely'});return
            changes={}
            for u,a,b in self.workspace.occurrences(resolved[0]):
                changes.setdefault(u,[]).append({'range':range_(self.workspace.docs[u],a,b),'newText':new})
            self.reply(id,{'changes':changes});return
        if method=='textDocument/signatureHelp':
            before=text[:at];match=re.search(r'([A-Za-z_]\w*)\s*\(([^()]*)$',before)
            if not match:self.reply(id,None);return
            name=match.group(1);candidates=[s for s in self.workspace.all_symbols() if s.kind=='method' and s.name==name]
            signatures=[{'label':s.name+'('+', '.join(t+' '+n for t,n,_,_ in s.params)+')','parameters':[{'label':t+' '+n} for t,n,_,_ in s.params]} for s in candidates]
            self.reply(id,{'signatures':signatures,'activeSignature':0,'activeParameter':match.group(2).count(',')} if signatures else None);return
        self.reply(id,None)
    def loop(self):
        while True:
            headers={}
            while True:
                line=sys.stdin.buffer.readline()
                if not line:return
                if line in (b'\r\n',b'\n'):break
                if b':' in line:
                    k,v=line.split(b':',1);headers[k.strip().lower()]=v.strip()
            length=int(headers.get(b'content-length',b'0'))
            if length<=0:continue
            msg=json.loads(sys.stdin.buffer.read(length))
            try:self.handle(msg)
            except SystemExit:raise
            except Exception as ex:
                if 'id' in msg:self.reply(msg['id'],error={'code':-32603,'message':str(ex)})
if __name__=='__main__':Server().loop()
