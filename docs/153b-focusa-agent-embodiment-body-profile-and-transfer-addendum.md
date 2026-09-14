# Spec 153B — Agent Embodiment, Body Profiles, and Cross-Body Continuity Addendum

**Status:** DRAFT canonical architecture direction — additive; does not supersede Specs 151, 153, 153A, 164, 181–184, or Veragensia Docs 191/193/195/197.  
**Owner:** Focusa Core / Physical and Scientific Cognition / Agent Capability Fabric  
**Created:** 2026-09-14  
**Purpose:** make explicit the embodiment model already implied by Focusa's whole-organism architecture, robotics/actuation primitives, and Veragensia body/runtime-transfer model.

---

## 0. One-line definition

> **An Agent Body is a replaceable execution embodiment attached to a persistent governed agent/partner identity. Changing bodies changes runtime incarnation, perception/actuation capability, trust, locality, and resource posture — not the persistent human-agent relationship, Workstream continuity, Foreman identity, accumulated Evidence, or canonical Focusa state.**

A Chromebook, desktop/laptop, cloud Agent Computer, mobile/wearable surface, embedded system, and future robotic or humanoid platform may all act as bodies or body-adjacent execution surfaces under this model.

The hardware is not the identity.

---

## 1. Why this addendum exists

Focusa already models:

```text
whole-organism cognition
sensor / perception
world-state estimate
controller
actuator
physical action
safety envelope
emergency stop
```

Veragensia already models one Agent Computer architecture across local and cloud bodies, runtime incarnations, resource replicas, enforcement, and state transfer.

The missing contract is the explicit bridge:

```text
persistent governed identity / relationship
                |
                v
          BodyBinding
                |
      +---------+----------+
      |         |          |
 digital body  cloud body  physical/robotic body
```

This addendum defines that bridge without moving primitive ownership.

---

## 2. Foundational laws

1. **Body is not identity.** A physical device, VM, browser profile, process, robot chassis, serial number, hostname, or runtime ID MUST NOT become the canonical identity of the persistent partner, Foreman, worker, or Workstream.
2. **Embodiment is replaceable.** A governed actor MAY detach from one body and attach to another while preserving higher-level continuity through existing Focusa identity/state primitives.
3. **A new body is a new runtime posture.** Body change requires fresh runtime incarnation, capability discovery, trust/attestation, placement, credential, resource, and enforcement evaluation.
4. **Capabilities come from the current body.** A body exposes what it can perceive, compute, communicate, and actuate. Capabilities do not follow identity merely because an earlier body had them.
5. **Relationship continuity is above hardware.** Human-agent relationship history, Workstream/Foreman continuity, accepted learning, Evidence, decisions, obligations, and current work remain in their canonical owners.
6. **Physical embodiment does not bypass governance.** A robotic actuator is an execution capability subject to the same authority, evidence, safety, consequence, and settlement discipline as any other consequential actuator.
7. **Hard-real-time control remains deterministic.** LLMs and high-level agents may plan, supervise, interpret, and steer; they MUST NOT directly own hard-real-time motor-control loops.
8. **Sensing does not imply permission.** A body possessing a camera, microphone, GPS, tactile sensor, browser, filesystem, or network interface does not grant Focusa or an agent ambient right to use it.
9. **Transfer never implies cloning authority.** Concurrent bodies require explicit topology/replica/control semantics; attaching a second body does not silently duplicate writer leases, credentials, actuator authority, or identity ownership.
10. **External effects survive body changes.** Migrating or replacing a body never rolls back real-world effects; settlement/reconciliation remains mandatory.

---

## 3. Body classes

Canonical body classes are intentionally capability-oriented rather than vendor-oriented.

```text
interactive_computer_body
cloud_agent_computer_body
mobile_or_wearable_body
embedded_body
robotic_body
humanoid_robotic_body
specialist_remote_body
```

A product profile MAY define additional classes, but they MUST map to the same body contract.

### 3.1 Interactive computer body

Examples: Chromebook, laptop, desktop, thin-client terminal.

Typical capability families:

```text
display
keyboard / pointer / touch
microphone / speaker
browser
local filesystem projection
terminal / developer tools
local application surfaces
local or remote agent interaction
```

### 3.2 Cloud Agent Computer body

Typical capability families:

```text
CPU / memory / storage
persistent or ephemeral desktop
browser/computer execution
Agent Apps
build/test/data workloads
remote human takeover
networked specialist services
```

### 3.3 Mobile/wearable body

Examples: phone, earbuds, watch, glasses.

Typical capability families:

```text
voice
camera
location/presence projection
notifications
trusted paired-device controls
portable conversation continuity
```

### 3.4 Robotic body

Typical capability families:

```text
vision / depth / lidar / proximity
microphone / speaker
proprioception
locomotion
manipulation
force / torque / tactile sensing
battery / thermal / physical-state telemetry
hardware emergency stop / watchdog
```

### 3.5 Humanoid robotic body

A humanoid is a specialization of `robotic_body`, not a new cognitive authority.

It MAY expose human-environment affordances such as:

```text
bipedal locomotion
human-scale reach
hands / grasping
head/torso orientation
speech and social presence
human tool / furniture compatibility
```

The humanoid form MUST NOT weaken the body-independent identity, permission, or safety model.

---

## 4. `AgentBodyProfile`

```yaml
schema: focusa.agent_body_profile.v1
body_profile_id:
body_class:
body_model_ref:
node_identity_ref:
runtime_incarnation_ref:
owner_principal_ref:

capabilities:
  perception: []
  actuation: []
  communication: []
  compute: []
  storage: []
  mobility: []
  human_interface: []

trust:
  trust_class:
  attestation_ref:
  enforcement_profile_ref:

resources:
  resource_budget_ref:
  energy_state_ref:
  thermal_state_ref:
  network_posture_ref:

physical:
  physical_system_ref:
  digital_twin_ref:
  safety_envelope_ref:
  emergency_stop_ref:

freshness_ref:
capability_snapshot_ref:
```

A profile describes current usable embodiment. It does not mint authority.

---

## 5. `BodyBinding`

```yaml
schema: focusa.body_binding.v1
body_binding_id:
actor_principal_ref:
agent_identity_ref:
role_ref:
workstream_root_ref:
foreman_ref:
body_profile_ref:
runtime_incarnation_ref:
placement_ref:
capability_projection_ref:
authority_projection_ref:
credential_posture_ref:
attached_at:
detached_at:
reason:
status: proposed | active | degraded | detached | fenced | unknown
```

Rules:

- `actor_principal_ref` / `agent_identity_ref` remain owned by existing identity primitives;
- `foreman_ref` remains the same when the same Workstream-scoped Foreman changes bodies;
- attaching a new body MUST NOT copy stale runtime handles from the previous body;
- detaching a body fences applicable actuator/control/credential leases;
- a body may be read-only, assistive, execution-capable, or physically actuating according to current grants.

---

## 6. Cross-body transfer

A body transfer is a governed re-binding operation, not a personality export.

```text
persistent identity + canonical state
            |
            v
    prepare transfer packet
            |
            v
  verify destination body
            |
            v
 fresh incarnation / trust / capabilities
            |
            v
 reissue only applicable grants / credentials
            |
            v
 attach same actor / Foreman / Workstream
            |
            v
 fence old body where required
```

The transfer packet SHOULD reference, rather than duplicate, canonical owners for:

```text
ProjectIdentity / WorkstreamRoot
ForemanBinding
Workpoint / Trajectory
active obligations / blockers
Conversation continuity
Evidence / Receipts
current assignments
approved learning
resource replicas / revisions
unsettled external effects
```

Body transfer MUST NOT rely on transcript replay as the sole continuity mechanism.

---

## 7. Body transfer versus body concurrency

These are distinct.

### Transfer

Primary execution presence moves from body A to body B.

### Concurrency

The same higher-level partner may have multiple simultaneous surfaces/bodies, for example:

```text
Chromebook        -> human interaction
Cloud Computer    -> heavy software execution
Phone / earbuds   -> ambient conversation
Humanoid body     -> physical presence / manipulation
```

Concurrency requires exact control/write/actuation ownership. Two bodies MUST NOT concurrently assume sole actuator authority over the same physical or software resource without an explicit multi-controller contract.

---

## 8. Capability adaptation after transfer

A body change may add or remove affordances.

Example:

```text
Chromebook body
  browser
  keyboard
  display
  microphone

        ↓ transfer / attach

Humanoid body
  vision
  speech
  locomotion
  manipulation
  tactile sensing
```

The system MUST re-resolve plans against the destination body's capability graph.

A prior workflow requiring `browser.navigate` cannot silently become `walk_to_terminal_and_click` unless a governed execution route explicitly resolves that substitution.

Likewise, a physical task requiring `manipulator.grasp` remains unavailable on a Chromebook body.

---

## 9. Robotic actuation integration

Spec 153 remains owner of physical modeling and measurement semantics. Spec 153A remains owner of consequential physical verification/settlement constraints. Spec 151 remains owner of capability/execution design.

A robotic action path is conceptually:

```text
human / mission intent
→ Focusa Workstream / Workpoint
→ Foreman / worker reasoning
→ registered physical operation
→ authority + capability + safety preflight
→ body capability binding
→ deterministic controller
→ actuator command
→ sensor / world observation
→ Evidence / settlement / reconciliation
```

Required before consequential physical actuation:

- exact body, actuator, and physical-system identity;
- current runtime incarnation;
- fresh sensor/calibration/world-state posture;
- applicable safety envelope;
- current authority/capability grant;
- deterministic control-path readiness;
- watchdog and emergency-stop readiness;
- bounded target/effect;
- post-effect observation and settlement.

---

## 10. Human-agent relationship continuity

Embodiment changes MUST preserve the distinction between:

```text
who the agent/partner is
what role it occupies
what project/workstream it is responsible for
what it currently knows through canonical state
what it is allowed to do
what body it currently inhabits
```

Only the final item is replaced by body transfer.

A customer moving from a Chromebook to a premium computer or humanoid body MUST NOT be treated as onboarding a new partner merely because the hardware changed.

---

## 11. Product trajectory

Focusa and Veragensia SHOULD remain body-agnostic enough that improvements in commodity hardware, cloud computers, wearables, robotics, and humanoids can be adopted without re-architecting the persistent relationship layer.

The long-term product trajectory explicitly includes **physical robotic and humanoid embodiments** as first-class future bodies for the same governed human-agent system.

This is a trajectory statement, not a claim that a humanoid hardware product is currently implemented or certified.

---

## 12. Acceptance invariants

A conforming implementation MUST be able to prove:

1. replacing hardware does not create a new canonical partner/Foreman identity;
2. destination-body capabilities are freshly discovered/verified;
3. stale runtime handles from the source body fail closed;
4. source-body actuator/control leases are fenced when transfer requires exclusivity;
5. Workstream/Workpoint/Evidence continuity survives transfer;
6. credential authority is re-evaluated rather than blindly copied;
7. external effects are reconciled rather than rolled back by transfer;
8. simultaneous bodies expose explicit control/write ownership;
9. physical actuation cannot execute without current safety/state authority;
10. a humanoid body remains one replaceable embodiment beneath the persistent relationship layer.

---

## 13. Cross-spec ownership map

| Concern | Canonical owner |
|---|---|
| Persistent agent identity / role | Focusa Spec 72 |
| Workstream continuity | Focusa Spec 164 |
| Project Foreman | Focusa Spec 182 |
| Program/capability design | Focusa Spec 151 |
| Physical system / sensor / actuator semantics | Focusa Spec 153 |
| Physical verification / consequential actuation | Focusa Spec 153A |
| Conversation continuity | Focusa Spec 181 |
| Presence / placement | Focusa Spec 139 |
| Credential authority | Focusa Spec 156 |
| Runtime body topology / cloud bodies | Veragensia Doc 191 |
| Machine enforcement | Veragensia Doc 193 |
| Human control / control leases | Veragensia Doc 194 |
| Runtime incarnation / state transfer | Veragensia Doc 195 |
| Platform/runtime trust | Veragensia Doc 196 |
| Voice-complete embodiment | Veragensia Doc 197 |
| Body portability bridge | **this addendum + Veragensia embodiment addendum** |

---

## 14. Non-goals

This addendum does not:

- select a humanoid manufacturer;
- define motor-control firmware;
- certify any robot for autonomous operation;
- move physical truth into an LLM;
- make Focusa a robotics middleware replacement;
- make body hardware the canonical identity;
- imply that current Focusa releases implement robotic-body attachment.

Its purpose is architectural: preserve one persistent governed human-agent system across replaceable digital, cloud, mobile, and physical bodies.
