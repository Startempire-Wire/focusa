#!/usr/bin/env python3
"""Private Google Messages connector for the Focusa SMS broker contract.

Runs inside the private browser trust boundary. Public responses use opaque
handles; browser selectors, cookies, profile state, and OTP values never leave.
"""
from __future__ import annotations
import hashlib,hmac,json,os,re,secrets,sys,threading,time,urllib.parse,urllib.request
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
import websocket

PORT=int(os.environ.get('FOCUSA_SMS_BROKER_PORT','8794')); CDP_PORT=int(os.environ.get('FOCUSA_SMS_CDP_PORT','9333')); TOKEN_FILE=os.environ.get('FOCUSA_SMS_BROKER_TOKEN_FILE','/tmp/focusa-sms-broker.token')
_token_stat=os.lstat(TOKEN_FILE)
if not os.path.isfile(TOKEN_FILE) or os.path.islink(TOKEN_FILE) or _token_stat.st_mode & 0o077: raise SystemExit('broker token permissions invalid')
TOKEN=open(TOKEN_FILE).read().strip();
if len(TOKEN)<32: raise SystemExit('broker token unavailable')
THREADS={}; CHALLENGES={}; EVENTS=[]; SENDS={}; LOCK=threading.RLock()

def envelope(ok,status,summary,**data): return {'schema':'focusa.tool_result_v1','canonical':True,'ok':ok,'status':status,'summary':summary,**data}
def handle(kind,value): return kind+'-'+hmac.new(TOKEN.encode(),value.encode(),hashlib.sha256).hexdigest()[:24]
def audit(action,status,failure_class=None):
 with LOCK:
  EVENTS.append({'schema':'focusa.sms_audit.v1','audit_id':secrets.token_hex(12),'action':action,'status':status,'failure_class':failure_class,'occurred_at':time.time()});del EVENTS[:-500]

class Cdp:
 def __init__(self):
  meta=json.load(urllib.request.urlopen(f'http://127.0.0.1:{CDP_PORT}/json/version',timeout=3));self.w=websocket.create_connection(meta['webSocketDebuggerUrl'],suppress_origin=True,timeout=10);self.i=0
 def call(self,m,p=None,s=None):
  self.i+=1;q={'id':self.i,'method':m,'params':p or {}};q.update({'sessionId':s}if s else{});self.w.send(json.dumps(q))
  while True:
   r=json.loads(self.w.recv())
   if r.get('id')==self.i:
    if 'error'in r:raise RuntimeError('cdp request rejected')
    if 'exceptionDetails'in r.get('result',{}):raise RuntimeError('page operation rejected')
    return r.get('result',{})
 def attach_messages(self):
  infos=self.call('Target.getTargets')['targetInfos']
  for item in infos:
   if item.get('type')!='page' or not item.get('browserContextId') or not item.get('url','').startswith('https://messages.google.com/'):continue
   session=self.call('Target.attachToTarget',{'targetId':item['targetId'],'flatten':True})['sessionId'];state=self.eval(session,"(()=>({paired:location.pathname.includes('/conversations'),unable:!!document.querySelector('mw-unable-to-connect-container'),list:!!document.querySelector('mws-conversations-list')}))()")
   if state.get('paired') and not state.get('unable') and state.get('list'): return session
  raise RuntimeError('paired connector unavailable')
 def eval(self,s,e):return self.call('Runtime.evaluate',{'expression':e,'returnByValue':True},s)['result'].get('value')
 def close(self):self.w.close()

def browser_health():
 c=Cdp()
 try:s=c.attach_messages();return c.eval(s,"({path:location.pathname,thread_count:document.querySelectorAll('mws-conversation-list-item').length})")
 finally:c.close()

def list_threads(limit):
 c=Cdp()
 try:
  s=c.attach_messages();rows=c.eval(s,"(()=>[...document.querySelectorAll('mws-conversation-list-item')].map((x,i)=>({index:i,name:(x.querySelector('h2.name')?.innerText||'').trim(),snippet:(x.querySelector('.snippet-text')?.innerText||'').trim(),timestamp:(x.querySelector('mws-relative-timestamp')?.innerText||'').trim(),unread:x.classList.contains('unread')||x.querySelector('[aria-label*=unread i]')!==null})))()") or []
  out=[]
  with LOCK:
   THREADS.clear()
   for row in rows[:limit]:
    token=handle('thread',f"{row['index']}\0{row['name']}");THREADS[token]=row['index'];out.append({'thread_handle':token,'display_name':row['name'],'snippet':row['snippet'],'relative_timestamp':row['timestamp'],'unread':row['unread']})
  return out
 finally:c.close()

def select_thread(thread_handle):
 with LOCK:index=THREADS.get(thread_handle)
 if index is None:list_threads(200);index=THREADS.get(thread_handle)
 if index is None:raise KeyError('thread handle unavailable')
 c=Cdp();s=c.attach_messages();ok=c.eval(s,f"(()=>{{const x=document.querySelectorAll('mws-conversation-list-item a.list-item')[{int(index)}];if(!x)return false;x.click();return true}})()")
 if not ok:c.close();raise RuntimeError('thread selection failed')
 time.sleep(1);return c,s

def read_messages(thread_handle,limit):
 c,s=select_thread(thread_handle)
 try:
  rows=c.eval(s,"(()=>[...document.querySelectorAll('[data-e2e-message],mws-message-wrapper,.message-row,.text-msg')].map((x,i)=>({index:i,body:(x.innerText||'').trim(),direction:x.classList.contains('outgoing')?'outgoing':x.classList.contains('incoming')?'incoming':'unknown'})).filter(x=>x.body))()") or []
  if not rows:raise RuntimeError('message DOM adapter unavailable')
  return [{'message_handle':handle('message',f"{thread_handle}\0{x['index']}\0{x['body']}"),'thread_handle':thread_handle,'direction':x['direction'],'body':x['body']} for x in rows[-limit:]]
 finally:c.close()

def send_message(thread_handle,body):
 c,s=select_thread(thread_handle)
 try:
  ok=c.eval(s,"(()=>{const x=document.querySelector('mws-message-compose textarea,textarea');if(!x)return false;x.focus();x.select();return true})()")
  if not ok:raise RuntimeError('compose input unavailable')
  c.call('Input.insertText',{'text':body},s);sent=c.eval(s,"(()=>{const x=document.querySelector('mws-message-send-button button,button[aria-label*=send i]');if(!x)return false;x.click();return true})()")
  if not sent:raise RuntimeError('send control unavailable')
  return handle('send',secrets.token_hex(16))
 finally:c.close()

class Handler(BaseHTTPRequestHandler):
 server_version='FocusaSmsBroker/1'
 def log_message(self,*args):pass
 def reply(self,status,value):
  data=json.dumps(value,separators=(',',':')).encode();self.send_response(status);self.send_header('content-type','application/json');self.send_header('cache-control','no-store');self.send_header('content-length',str(len(data)));self.end_headers();self.wfile.write(data)
 def auth(self):return hmac.compare_digest(self.headers.get('authorization',''),f'Bearer {TOKEN}')
 def body(self):
  size=int(self.headers.get('content-length','0'));return json.loads(self.rfile.read(size)) if 0<size<=65536 else {}
 def route(self):return urllib.parse.urlsplit(self.path)
 def do_GET(self):
  if not self.auth():return self.reply(401,envelope(False,'blocked','Broker authorization required',failure_class='unauthorized'))
  u=self.route();q=urllib.parse.parse_qs(u.query)
  try:
   if u.path=='/v1/sms/health':
    h=browser_health();return self.reply(200,envelope(True,'ready_live_source','SMS connector ready; encrypted restore remains degraded',connector={'connector_id':'google-messages-1','kind':'google_messages','paired':True,'thread_count':h['thread_count'],'checkpoint_status':'degraded'}))
   if u.path=='/v1/sms/enrollment':return self.reply(200,envelope(True,'paired_live_source','Customer-owned connector paired; durable restore repair required',connector_id='google-messages-1'))
   if u.path=='/v1/sms/threads':
    rows=list_threads(min(int(q.get('limit',['50'])[0]),200));audit('list_threads','ok');return self.reply(200,envelope(True,'ok','Authorized thread summaries',threads=rows))
   m=re.fullmatch(r'/v1/sms/threads/([A-Za-z0-9_.:-]+)/messages',u.path)
   if m:
    rows=read_messages(m.group(1),min(int(q.get('limit',['50'])[0]),200));audit('read_thread','ok');return self.reply(200,envelope(True,'ok','Authorized bounded thread read',messages=rows))
   if u.path=='/v1/sms/search':
    needle=q.get('query',[''])[0].casefold();rows=[x for x in list_threads(200) if needle in (x['display_name']+' '+x['snippet']).casefold()][:min(int(q.get('limit',['50'])[0]),200)];audit('search','ok');return self.reply(200,envelope(True,'ok','Authorized bounded search',matches=rows))
   if u.path=='/v1/sms/events':
    with LOCK:rows=list(EVENTS[-min(int(q.get('limit',['100'])[0]),500):])
    return self.reply(200,envelope(True,'ok','Value-free broker events',events=rows))
   return self.reply(404,envelope(False,'blocked','Unknown SMS route',failure_class='not_found'))
  except KeyError:return self.reply(404,envelope(False,'blocked','Opaque handle is unavailable',failure_class='handle_unavailable'))
  except Exception as error:audit(u.path,'blocked',type(error).__name__);return self.reply(503,envelope(False,'degraded','Connector operation unavailable',failure_class='connector_operation_unavailable'))
 def do_POST(self):
  if not self.auth():return self.reply(401,envelope(False,'blocked','Broker authorization required',failure_class='unauthorized'))
  u=self.route();b=self.body()
  try:
   if u.path=='/v1/sms/send':
    if not b.get('confirm',False):return self.reply(403,envelope(False,'blocked','Send requires confirm=true',failure_class='approval_required'))
    recipients=b.get('recipient_handles') or [];body=str(b.get('body',''));idem=str(b.get('idempotency_key',''));grant=str(b.get('grant_id',''));consumer=str(b.get('consumer_ref',''))
    if len(recipients)!=1 or not body.strip() or not idem or not grant or not consumer:return self.reply(400,envelope(False,'blocked','Invalid bounded send request',failure_class='validation_rejected'))
    with LOCK: prior=SENDS.get((grant,consumer,idem))
    if prior:return self.reply(200,envelope(True,'sent','Idempotent send receipt replayed',send_handle=prior,idempotency_key=idem,replayed=True))
    receipt=send_message(recipients[0],body)
    with LOCK:SENDS[(grant,consumer,idem)]=receipt
    audit('send','ok');return self.reply(200,envelope(True,'sent','Message sent',send_handle=receipt,idempotency_key=idem,replayed=False))
   if u.path=='/v1/sms/otp/challenges':
    required=('provider','target_handle','consumer_ref');
    if any(not b.get(x) for x in required):return self.reply(400,envelope(False,'blocked','Challenge scope incomplete',failure_class='validation_rejected'))
    h=handle('challenge',secrets.token_hex(16));ttl=min(max(int(b.get('ttl_seconds',300)),30),600)
    with LOCK:CHALLENGES[h]={'provider':b['provider'],'target_handle':b['target_handle'],'consumer_ref':b['consumer_ref'],'expires':time.time()+ttl,'status':'waiting'}
    audit('otp_challenge','ok');return self.reply(200,envelope(True,'waiting','OTP challenge registered',challenge_handle=h,expires_in_seconds=ttl))
   if u.path=='/v1/sms/otp/inject':return self.reply(501,envelope(False,'blocked','Bound target injection adapter not registered',failure_class='target_adapter_unavailable'))
   if u.path=='/v1/sms/checkpoint':return self.reply(409,envelope(False,'degraded','Live source remains paired; independent restore repair required',failure_class='restored_connector_unavailable'))
   if u.path=='/v1/sms/revoke':return self.reply(403,envelope(False,'blocked','Revoke requires protected owner workflow',failure_class='owner_workflow_required'))
   return self.reply(404,envelope(False,'blocked','Unknown SMS route',failure_class='not_found'))
  except Exception as error:audit(u.path,'blocked',type(error).__name__);return self.reply(503,envelope(False,'degraded','Connector operation unavailable',failure_class='connector_operation_unavailable'))

def main():
 ThreadingHTTPServer(('0.0.0.0',PORT),Handler).serve_forever()

if __name__=='__main__':main()
