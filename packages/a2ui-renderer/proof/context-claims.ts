import { createFocusaSpec135Client, type components } from "@focusa/spec135-client";
import actionBindings from "../../../docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json" with { type: "json" };
import { FOCUSA_A2UI_CATALOG_ID, FocusaA2uiRenderer, type A2uiClientAction, type A2uiMessage } from "../src/index.js";

type Mutation = components["schemas"]["focusa_context_graph_mutation_request_v1"];
const scope={project_root:"/example/focusa",continuity_id:"focusa-cont-c3-generated-ui",attachment_id:"attachment:c3-context"};
const binding=actionBindings.bindings.find(candidate=>candidate.action_id==="focusa.context.graph.mutate");
if(!binding||!binding.control.idempotency_required||!binding.control.receipt_required)throw new Error("Generated Context graph binding unavailable");
const client=createFocusaSpec135Client({baseUrl:window.location.origin});
const surface=document.querySelector<HTMLElement>("#context-claim-surface")!;const status=document.querySelector<HTMLElement>("#claim-result")!;
const observedActions:A2uiClientAction[]=[];const responses:Array<components["schemas"]["focusa_context_graph_mutation_result_v1"]>=[];let lastError:unknown;

const renderer=new FocusaA2uiRenderer({allowedActionNames:new Set([binding.action_id]),async onAction(action){
 observedActions.push(action);status.textContent="Creating source-linked candidate claims…";renderer.processDelta(progress("Proposing candidate claims",20));
 try{
  const read=await client.GET("/v1/context/graph",{params:{query:scope}});if(read.error||!read.data)throw read.error??new Error("Context graph unavailable");let version=read.data.state_version;
  const mutate=async(body:Omit<Mutation,"expected_state_version">)=>{const response=await client.POST("/v1/context/graph/mutate",{params:{query:scope},body:{...body,expected_state_version:version}});if(response.error||!response.data)throw response.error??new Error("Context graph mutation unavailable");responses.push(response.data);version=response.data.state_version;return response.data};
  const first=await mutate({...scope,idempotency_key:"c3-ui-claim-a",action:"propose_claim",claim:"Release artifacts require signatures.",source_citation_refs:["citation:669f3c8e99066a516baf278f"],confidence:.92});
  const left=first.claims.find(claim=>claim.idempotency_key==="c3-ui-claim-a")!;
  const second=await mutate({...scope,idempotency_key:"c3-ui-claim-b",action:"propose_claim",claim:"Release artifacts do not require signatures.",source_citation_refs:["citation:142a5009211501ec2d5f602e"],confidence:.88});
  const right=second.claims.find(claim=>claim.idempotency_key==="c3-ui-claim-b")!;
  renderer.processDelta(progress("Blocking contradictory claims",55));
  const opened=await mutate({...scope,idempotency_key:"c3-ui-open",action:"open_contradiction",left_claim_id:left.claim_id,right_claim_id:right.claim_id,rationale:"Retrieved sources make opposite release assertions."});
  const edge=opened.contradictions.find(candidate=>candidate.status==="open")!;
  renderer.processDelta(progress("Recording explicit operator resolution",82));
  const resolved=await mutate({...scope,idempotency_key:"c3-ui-resolve",action:"resolve_contradiction",contradiction_id:edge.contradiction_id,resolution:"accept_left",selected_claim_id:left.claim_id,actor:"operator",rationale:"The signed release policy is authoritative.",source_citation_refs:left.source_citation_refs});
  const accepted=resolved.claims.find(claim=>claim.claim_id===left.claim_id)!;const rejected=resolved.claims.find(claim=>claim.claim_id===right.claim_id)!;
  renderer.processDelta([{version:"v0.9",updateComponents:{surfaceId:"c3-context-claims",components:[
   {id:"progress",component:"FocusaProgressStepper",label:"Context claim review complete",description:`Reactive projection revision ${resolved.projection.revision} is unblocked.`,status:"completed",progress:100},
   {id:"claim",component:"FocusaContextClaimReview",label:"Accepted source-linked claim",description:accepted.claim,status:"accepted",details:`claim=${accepted.claim_id}; citations=${accepted.source_citation_refs.join(", ")}; confidence=${accepted.confidence}`},
   {id:"contradiction",component:"FocusaContradictionCard",label:"Contradiction resolved",description:`${rejected.claim} was rejected by explicit resolution.`,status:"resolved",details:`edge=${edge.contradiction_id}; accepted=${accepted.claim_id}; rejected=${rejected.claim_id}`},
   {id:"approval",component:"FocusaApprovalCard",label:"Operator decision recorded",description:resolved.decisions.at(-1)?.rationale??"Resolution recorded",status:"approved",details:`decision=${resolved.decisions.at(-1)?.decision_id}; projection=${JSON.stringify(resolved.projection)}`},
   {id:"evidence",component:"FocusaEvidenceSummary",label:"Canonical claim graph Evidence",description:`Evidence ${resolved.evidence_ref}`,status:"saved",details:`claims=${resolved.claims.length}; decisions=${resolved.decisions.length}`},
   {id:"receipt",component:"FocusaReceiptCard",label:"Context graph Receipt",description:resolved.receipt_ref,status:"committed",details:`operation=${binding.action_id}; state_version=${resolved.state_version}`}
  ]}}]);status.textContent="Context contradiction resolved and reactive truth updated";document.body.dataset.claimStatus="completed";
 }catch(error){lastError=error;status.textContent="Context claim review needs recovery";renderer.processDelta([{version:"v0.9",updateComponents:{surfaceId:"c3-context-claims",components:[{id:"evidence",component:"FocusaRecoveryCard",label:"Context graph needs recovery",description:"Reload exact scope and retry idempotently.",status:"retry",details:JSON.stringify(error)}]}}]);document.body.dataset.claimStatus="recovery";}
}});
function progress(label:string,value:number):A2uiMessage[]{return[{version:"v0.9",updateComponents:{surfaceId:"c3-context-claims",components:[{id:"progress",component:"FocusaProgressStepper",label,description:"Claims remain source-linked and noncanonical until explicit review.",status:"processing",progress:value}]}}]}
const snapshot:A2uiMessage[]=[{version:"v0.9",createSurface:{surfaceId:"c3-context-claims",catalogId:FOCUSA_A2UI_CATALOG_ID}},{version:"v0.9",updateComponents:{surfaceId:"c3-context-claims",components:[
 {id:"root",component:"Column",children:["stage","progress","review","claim","contradiction","approval","evidence","receipt"]},
 {id:"stage",component:"FocusaStageShell",label:"Review reactive Context",description:"Promote source-linked claims only through explicit decisions.",status:"ready",details:`operation=${binding.action_id}; scope=${JSON.stringify(scope)}`},
 {id:"progress",component:"FocusaProgressStepper",label:"Claims ready",description:"No graph mutation has run yet.",status:"ready",progress:0},
 {id:"review",component:"FocusaPrimaryAction",label:"Resolve source contradiction",description:"Creates canonical claim, contradiction, decision, projection, Evidence, and Receipt records.",primaryActionLabel:"Review Context Claims",action:{event:{name:binding.action_id,context:scope}}},
 {id:"claim",component:"FocusaContextClaimReview",label:"Candidate claims",description:"Source-linked claims will appear here.",status:"pending"},
 {id:"contradiction",component:"FocusaContradictionCard",label:"Contradiction review",description:"Open contradiction edges block reactive acceptance.",status:"pending"},
 {id:"approval",component:"FocusaApprovalCard",label:"Approval state",description:"Explicit operator decisions will appear here.",status:"pending"},
 {id:"evidence",component:"FocusaEvidenceSummary",label:"Claim graph proof",description:"Evidence will appear here.",status:"pending"},
 {id:"receipt",component:"FocusaReceiptCard",label:"Mutation receipt",description:"Receipt will appear here.",status:"pending"}
]}}];renderer.processSnapshot(snapshot);renderer.mount(surface,"c3-context-claims");Object.assign(window,{focusaContextClaimEval:{renderer,binding,scope,observedActions,responses,get lastError(){return lastError}}});
