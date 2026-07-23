#!/usr/bin/env python3
"""Deterministic Spec133 subprocess/fault fixture; invoked by the final runtime matrix."""
import argparse, json, os, subprocess, sys, tempfile, time
p=argparse.ArgumentParser(); p.add_argument('mode', choices=['harness','subprocess','child-leak','prompt-wait','output-flood','model-mismatch','retry-failure','isolated-git','entitlement','runner-disconnect']); p.add_argument('--lines',type=int,default=32); a=p.parse_args()
def emit(kind,**fields): print(json.dumps({'kind':kind,**fields}),flush=True)
if a.mode=='harness': emit('harness.ready',capabilities=['stream','control','model-observation'])
elif a.mode=='subprocess': emit('process.started',pid=os.getpid()); emit('process.exited',code=0)
elif a.mode=='child-leak':
 c=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'],start_new_session=False); emit('child.spawned',pid=c.pid); c.terminate(); c.wait(timeout=5); emit('child.cleaned',pid=c.pid)
elif a.mode=='prompt-wait': emit('waiting_input',prompt='fixture approval required')
elif a.mode=='output-flood':
 for i in range(min(a.lines,1000)): emit('output',sequence=i,payload='x'*128)
elif a.mode=='model-mismatch': emit('model.observed',requested='provider/model-a',observed='provider/model-b')
elif a.mode=='retry-failure':
 for i in range(3): emit('provider.failure',attempt=i+1,retryable=i<2)
elif a.mode=='isolated-git':
 with tempfile.TemporaryDirectory() as d: emit('workspace.isolated',path=d,clean=True)
elif a.mode=='entitlement': emit('entitlement.checked',model='provider/model-a',entitled=True)
elif a.mode=='runner-disconnect': emit('runner.disconnected',reconnectable=True); emit('runner.reconnected',authenticated=True)
