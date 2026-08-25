import test from 'node:test';
import assert from 'node:assert/strict';
import { notificationFromEvent, unreadNotificationCount } from '../src/lib/notifications.mjs';

const event={schema:'focusa.stream_event.v1',event_id:'evt-1',cursor:'4',timestamp:'2026-08-24T12:00:00Z',event_type:'work_blocked',schema_version:'1',sequence:4,scope:{organization_id:'org-demo'},invalidate:[],payload:{summary:'Approval is required',secret:'do-not-store'}};

test('important events become safe bounded notifications without raw payload',()=>{
  const notification=notificationFromEvent(event);
  assert.equal(notification.title,'work blocked');
  assert.equal(notification.body,'Approval is required');
  assert.equal(notification.source,'org-demo');
  assert.doesNotMatch(JSON.stringify(notification),/secret|do-not-store/);
});

test('ordinary stream events stay out of notification center',()=>{
  assert.equal(notificationFromEvent({...event,event_type:'projection_refreshed',payload:{summary:'routine'}}),null);
});

test('unread count is explicit and read state is not inferred from severity',()=>{
  assert.equal(unreadNotificationCount([{read:false},{read:true},{read:false}]),2);
});
