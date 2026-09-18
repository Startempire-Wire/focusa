import hashlib, json, os, pathlib, socket, subprocess, sys, tempfile, time, urllib.request, urllib.parse

binary = pathlib.Path(sys.argv[1]).resolve()
expected = sys.argv[2]
actual = hashlib.sha256(binary.read_bytes()).hexdigest()
assert actual == expected, (actual, expected)
root = pathlib.Path(tempfile.mkdtemp(prefix='focusa-621-http-'))
print('evidence_directory=' + str(root), flush=True)
print('daemon_sha256=' + actual, flush=True)
projects = [root / 'project-a', root / 'project-b']
for index, project in enumerate(projects):
    project.mkdir()
    subprocess.run(['git', 'init', '--quiet', str(project)], check=True)
    (project / '.focusa-project.json').write_text(json.dumps({
        'schema': 'focusa.project_marker.v1', 'project_id': 'issue621-' + str(index),
        'canonical_name': 'Issue621 fixture ' + str(index), 'project_root': str(project)
    }))
with socket.socket() as listener:
    listener.bind(('127.0.0.1', 0))
    port = listener.getsockname()[1]
base = 'http://127.0.0.1:' + str(port)
env = dict(os.environ, FOCUSA_BIND='127.0.0.1:' + str(port), FOCUSA_DATA_DIR=str(root / 'data'), FOCUSA_TEST_MODE='1')
log = (root / 'daemon.log').open('wb')
child = subprocess.Popen([str(binary)], env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)

def request(name, path, project, body=None):
    headers = {'Content-Type': 'application/json', 'X-Scope-Project-Root': str(project), 'X-Scope-Continuity-Id': 'issue621-http'}
    req = urllib.request.Request(base + path, data=None if body is None else json.dumps(body).encode(), headers=headers)
    with urllib.request.urlopen(req, timeout=30) as response:
        value = json.load(response)
    (root / (name + '.json')).write_text(json.dumps(value, indent=2))
    return value

try:
    deadline = time.monotonic() + 30
    while True:
        assert child.poll() is None, 'fixture daemon exited; inspect daemon.log'
        try:
            with socket.create_connection(('127.0.0.1', port), timeout=.2):
                break
        except OSError:
            assert time.monotonic() < deadline, 'fixture readiness timeout'
            time.sleep(.1)
    project = projects[0]
    body = dict(project_root=str(project), continuity_id='issue621-http',
                long_term_goal='Preserve scoped trajectory authority',
                desired_end_state='Committed goals remain readable after checkpoint',
                mid_level_goal='Verify saved goal visibility', short_term_goal='Prove HTTP readback',
                current_state='Isolated fixture with verified local identity',
                operator_confirmed=True, current_ask='Verify isolated trajectory roundtrip')
    defined = request('define', '/v1/trajectory/define-goal', project, body)
    assert defined.get('canonical') is True and defined.get('persisted') is True, defined
    checkpoint = request('checkpoint', '/v1/trajectory/checkpoint', project,
                         dict(project_root=str(project), continuity_id='issue621-http', summary='HTTP roundtrip'))
    assert checkpoint.get('persisted') is True, checkpoint
    def view(name, target):
        query = urllib.parse.urlencode(dict(project_root=str(target), continuity_id='issue621-http'))
        return request(name, '/v1/trajectory/view?' + query, target)
    same = view('same-scope-view', project)['trajectory']
    assert same['long_term_goal'] == body['long_term_goal'], same
    assert same['trajectory_ladder']['mlg'] == body['mid_level_goal'], same
    assert same['trajectory_ladder']['stg'] == body['short_term_goal'], same
    assert same['durable_lifecycle']['checkpoint_count'] >= 1, same
    history_query = urllib.parse.urlencode(dict(project_root=str(project), continuity_id='issue621-http', trajectory_id=defined['trajectory_id']))
    history = request('trajectory-history', '/v1/trajectory/history?' + history_query, project)
    assert history['reconstruction']['hlt'] == same['long_term_goal'], history
    assert history['reconstruction']['mlg'] == same['trajectory_ladder']['mlg'], history
    assert history['reconstruction']['stg'] == same['trajectory_ladder']['stg'], history
    versions = {event['hlt_version'] for event in history['events']}
    assert len(versions) == 1, versions
    version = versions.pop()
    assert same.get('hlt_version') == version, {'view_hlt_version': same.get('hlt_version'), 'ledger_hlt_version': version}
    foreign = view('foreign-scope-view', projects[1])['trajectory']
    assert foreign.get('long_term_goal') is None, foreign
    assert foreign['durable_lifecycle']['canonical'] is False, foreign
    repeat = request('second-checkpoint', '/v1/trajectory/checkpoint', project,
                     dict(project_root=str(project), continuity_id='issue621-http',
                          summary='Second checkpoint', idempotency_key='different-checkpoint-key'))
    assert repeat['trajectory_checkpoint']['trajectory_id'] == defined['trajectory_id'], repeat
    child.terminate()
    child.wait(timeout=10)
    child = subprocess.Popen([str(binary)], env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
    deadline = time.monotonic() + 30
    while True:
        assert child.poll() is None, 'restarted fixture exited; inspect daemon.log'
        try:
            with socket.create_connection(('127.0.0.1', port), timeout=.2):
                break
        except OSError:
            assert time.monotonic() < deadline, 'restart readiness timeout'
            time.sleep(.1)
    replay = view('restarted-view', project)['trajectory']
    assert replay['long_term_goal'] == body['long_term_goal'], replay
    assert replay['durable_lifecycle']['checkpoint_count'] >= 2, replay
    print('PASS: HTTP define/checkpoint/view, foreign-project isolation and restart readback', flush=True)
finally:
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=10)
    log.close()
    print('fixture_exit=' + str(child.returncode) + '; evidence retained=' + str(root), flush=True)
