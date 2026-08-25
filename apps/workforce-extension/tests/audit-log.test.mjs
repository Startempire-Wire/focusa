import test from 'node:test';
import assert from 'node:assert/strict';
import { auditRecordFromEvent } from '../src/lib/audit-log.mjs';

const event={schema:'focusa.stream_event.v1',event_id:'evt-2',cursor:'9',timestamp:'2026-08-24T12:00:00Z',event_type:'work_completed',schema_version:'1',sequence:9,correlation_id:'corr-1',scope:{organization_id:'org-demo',project_root:'/private'},invalidate:['work_loop'],payload:{secret:'never persist'}};

test('audit record preserves bounded provenance and excludes payload',()=>{
  const record=auditRecordFromEvent(event,'KH daemon');
  assert.deepEqual(record,{event_id:'evt-2',cursor:'9',timestamp:'2026-08-24T12:00:00Z',event_type:'work_completed',schema_version:'1',correlation_id:'corr-1',invalidate:['work_loop'],source:'KH daemon',scope_keys:['organization_id','project_root']});
  assert.doesNotMatch(JSON.stringify(record),/private|secret|never persist/);
});

test('malformed or non-Focusa events fail closed',()=>{
  assert.equal(auditRecordFromEvent({event_type:'work_completed'}),null);
  assert.equal(auditRecordFromEvent(null),null);
});
