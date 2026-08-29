#!/usr/bin/env python3
"""Regression checks for Plan 180 private broker and public adapters."""
from __future__ import annotations
import os,runpy,tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
BROKER=ROOT/'scripts/focusa-google-messages-broker.py'

with tempfile.TemporaryDirectory(prefix='focusa-sms-broker-test-') as raw:
    token=Path(raw)/'token'
    token.write_text('t'*64)
    token.chmod(0o600)
    previous=os.environ.get('FOCUSA_SMS_BROKER_TOKEN_FILE')
    os.environ['FOCUSA_SMS_BROKER_TOKEN_FILE']=str(token)
    try:
        module=runpy.run_path(str(BROKER),run_name='focusa_sms_broker_test')
    finally:
        if previous is None:
            os.environ.pop('FOCUSA_SMS_BROKER_TOKEN_FILE',None)
        else:
            os.environ['FOCUSA_SMS_BROKER_TOKEN_FILE']=previous

first=module['handle']('thread','provider-internal-value')
second=module['handle']('thread','provider-internal-value')
assert first==second and first.startswith('thread-')
assert 'provider-internal-value' not in first

receipt=module['envelope'](True,'ok','bounded')
assert receipt=={
    'schema':'focusa.tool_result_v1',
    'canonical':True,
    'ok':True,
    'status':'ok',
    'summary':'bounded',
}
module['audit']('health','ok')
event=module['EVENTS'][-1]
assert set(event)=={'schema','audit_id','action','status','failure_class','occurred_at'}
assert not any(key in event for key in ('body','otp','token','cookie','selector'))

api=(ROOT/'crates/focusa-api/src/routes/sms.rs').read_text()
assert 'sms_broker_url_not_private' in api
assert 'sms_broker_token_permissions_invalid' in api
assert '.content_length()' in api and api.count('1_048_576') >= 2
cli=(ROOT/'crates/focusa-cli/src/commands/sms.rs').read_text()
assert 'io::stdin().read_to_string' in cli
assert 'send requires --confirm' in cli
pi=(ROOT/'apps/pi-extension/src/sms-tools.ts').read_text()
for name in ('focusa_sms_health','focusa_sms_enrollment','focusa_sms_threads','focusa_sms_read_thread','focusa_sms_search','focusa_sms_send','focusa_sms_otp_challenge','focusa_sms_otp_inject','focusa_sms_checkpoint','focusa_sms_events','focusa_sms_revoke'):
    assert name in pi,name
assert 'The OTP value never enters model context' in pi
print('sms broker contract: passed')
